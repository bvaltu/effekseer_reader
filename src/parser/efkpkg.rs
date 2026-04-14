//! Parser for `.efkpkg` package files.
//!
//! An `.efkpkg` is a standard ZIP archive produced by the Effekseer editor's
//! *Export Package* command. It bundles a primary effect (`.efkefc`) together
//! with every texture, model, sound, material, and curve the effect references,
//! keyed by content hash so duplicates are deduplicated automatically.
//!
//! Inside the zip:
//! - `metafile.json` — index describing each entry (see [`Metafile`] below).
//! - One entry per file, named `<MD5-hex>-<LEN-hex>`.
//!
//! The primary effect's internal resource references are rewritten at export
//! time to the hash names, so looking up a referenced resource means calling
//! [`EfkPkg::get`] with the string that appears in the parsed effect (rather
//! than its original filesystem path).

use std::collections::HashMap;
use std::io::{Cursor, Read};

use serde::Deserialize;

use crate::error::Error;
use crate::types::ParseConfig;
use crate::types::effect::Effect;

use super::efkefc;
use super::effect;

/// Type of a file bundled inside an `.efkpkg`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum EfkPkgFileType {
    /// An `.efkefc` effect.
    Effect,
    /// A texture (image bytes in the original format, typically PNG/DDS).
    Texture,
    /// An audio file.
    Sound,
    /// An `.efkmodel` model.
    Model,
    /// An `.efkmat` material.
    Material,
    /// A NURBS curve file.
    Curve,
}

/// A single file entry extracted from an `.efkpkg`.
#[derive(Debug, Clone)]
pub struct EfkPkgFile {
    /// What kind of file this is.
    pub file_type: EfkPkgFileType,
    /// Original on-disk relative path at export time (e.g. `"textures/foo.png"`).
    /// Useful for deriving the file extension / MIME type.
    pub relative_path: String,
    /// Content-hash name — this is the key used inside the zip and the string
    /// the bundled effect's resource fields reference.
    pub hash_name: String,
    /// Raw undecoded bytes. Parsing (image decode, material parse, etc.) is
    /// deferred to the caller.
    pub data: Vec<u8>,
    /// Hash names of files this entry depends on (e.g. a material's textures).
    /// Only populated when the editor recorded dependencies.
    pub dependencies: Vec<String>,
}

/// A parsed `.efkpkg` package held entirely in memory.
#[derive(Debug, Clone)]
pub struct EfkPkg {
    /// Editor version that produced the package.
    pub version: String,
    /// The primary bundled effect (first `Effect`-typed entry).
    pub effect: EfkPkgFile,
    /// Additional effects, if the package contains more than one.
    pub extra_effects: Vec<EfkPkgFile>,
    /// All bundled files keyed by [`EfkPkgFile::hash_name`].
    pub files: HashMap<String, EfkPkgFile>,
}

impl EfkPkg {
    /// Look up a file by its hash name (the form embedded in the effect's
    /// resource references).
    pub fn get(&self, hash_name: &str) -> Option<&EfkPkgFile> {
        self.files.get(hash_name)
    }

    /// Find a file by its original relative path. Linear scan; intended for
    /// tooling rather than hot paths.
    pub fn find_by_relative_path(&self, path: &str) -> Option<&EfkPkgFile> {
        self.files.values().find(|f| f.relative_path == path)
    }

    /// Parse the primary bundled effect using the default strict config.
    pub fn parse_effect(&self) -> Result<Effect, Error> {
        self.parse_effect_with_config(&ParseConfig::default())
    }

    /// Parse the primary bundled effect with a custom config.
    pub fn parse_effect_with_config(&self, config: &ParseConfig) -> Result<Effect, Error> {
        parse_bundled_effect(&self.effect.data, config)
    }
}

// ─── Metafile schema ───

#[derive(Deserialize)]
struct Metafile {
    version: String,
    files: HashMap<String, MetafileEntry>,
}

#[derive(Deserialize)]
struct MetafileEntry {
    #[serde(rename = "type")]
    file_type: EfkPkgFileType,
    #[serde(default, alias = "name")]
    relative_path: String,
    #[serde(default)]
    dependencies: Vec<String>,
}

// ─── Entry points ───

/// Parse an `.efkpkg` package from its raw bytes using default strict config.
///
/// The returned [`EfkPkg`] holds the primary effect plus every bundled resource
/// as raw bytes. Resources are not decoded eagerly; parse them on demand via
/// [`EfkPkg::parse_effect`] or by feeding [`EfkPkgFile::data`] to the matching
/// `load_*` function.
///
/// # Example
///
/// ```no_run
/// let bytes = std::fs::read("bundle.efkpkg").unwrap();
/// let pkg = effekseer_reader::load_efkpkg(&bytes).unwrap();
/// let effect = pkg.parse_effect().unwrap();
/// for path in &effect.color_images {
///     if let Some(file) = pkg.get(path) {
///         // `file.data` contains the original texture bytes
///         // `file.relative_path` preserves the original filename/extension
///         let _ = file;
///     }
/// }
/// ```
pub fn load_efkpkg(data: &[u8]) -> Result<EfkPkg, Error> {
    load_efkpkg_with_config(data, &ParseConfig::default())
}

/// Parse an `.efkpkg` package from its raw bytes.
///
/// The `config` parameter is currently unused during package extraction itself
/// but is accepted for API symmetry with the other `load_*_with_config` calls
/// and forwarded to [`EfkPkg::parse_effect_with_config`] when the caller later
/// parses the bundled effect.
pub fn load_efkpkg_with_config(data: &[u8], _config: &ParseConfig) -> Result<EfkPkg, Error> {
    let mut archive = zip::ZipArchive::new(Cursor::new(data))
        .map_err(|e| Error::ZipError(e.to_string()))?;

    let metafile = read_metafile(&mut archive)?;

    let mut files: HashMap<String, EfkPkgFile> = HashMap::with_capacity(metafile.files.len());
    let mut effect_hashes: Vec<String> = Vec::new();

    for (hash_name, entry) in metafile.files {
        if hash_name == "metafile.json" || hash_name.contains('/') || hash_name.contains('\\') {
            return Err(Error::ZipError(format!(
                "suspicious entry name in metafile: {hash_name}"
            )));
        }

        let bytes = read_entry(&mut archive, &hash_name)?;

        if entry.file_type == EfkPkgFileType::Effect {
            effect_hashes.push(hash_name.clone());
        }

        files.insert(
            hash_name.clone(),
            EfkPkgFile {
                file_type: entry.file_type,
                relative_path: entry.relative_path,
                hash_name,
                data: bytes,
                dependencies: entry.dependencies,
            },
        );
    }

    let Some(primary_hash) = effect_hashes.first().cloned() else {
        return Err(Error::NoEffectInPackage);
    };
    let effect = files
        .get(&primary_hash)
        .expect("primary effect was just inserted")
        .clone();
    let extra_effects: Vec<EfkPkgFile> = effect_hashes
        .into_iter()
        .skip(1)
        .map(|h| files.get(&h).expect("extra effect hash indexed above").clone())
        .collect();

    Ok(EfkPkg {
        version: metafile.version,
        effect,
        extra_effects,
        files,
    })
}

// ─── Internals ───

fn read_metafile<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Result<Metafile, Error> {
    let mut entry = match archive.by_name("metafile.json") {
        Ok(e) => e,
        Err(zip::result::ZipError::FileNotFound) => return Err(Error::MissingMetafile),
        Err(e) => return Err(Error::ZipError(e.to_string())),
    };
    let mut buf = Vec::with_capacity(entry.size() as usize);
    entry
        .read_to_end(&mut buf)
        .map_err(|e| Error::ZipError(e.to_string()))?;
    // The Effekseer editor writes metafile.json via .NET's StreamWriter with
    // UTF-8 encoding, which emits a BOM (EF BB BF) by default. serde_json does
    // not strip BOMs, so do it here.
    let body = buf.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(&buf);
    serde_json::from_slice(body).map_err(|e| Error::JsonError(e.to_string()))
}

fn read_entry<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> Result<Vec<u8>, Error> {
    let mut entry = match archive.by_name(name) {
        Ok(e) => e,
        Err(zip::result::ZipError::FileNotFound) => {
            return Err(Error::MissingEntry(name.to_string()));
        }
        Err(e) => return Err(Error::ZipError(e.to_string())),
    };
    let mut buf = Vec::with_capacity(entry.size() as usize);
    entry
        .read_to_end(&mut buf)
        .map_err(|e| Error::ZipError(e.to_string()))?;
    Ok(buf)
}

fn parse_bundled_effect(data: &[u8], config: &ParseConfig) -> Result<Effect, Error> {
    // Bundled effects are full `.efkefc` containers (with EDIT chunk stripped-or-kept
    // by the exporter depending on version). Reuse the existing dispatch.
    let chunks = efkefc::extract_chunks(data)?;
    let target_location = if let Some(edit_data) = chunks.edit {
        match efkefc::parse_edit_chunk(edit_data) {
            Ok(b) => b.target_location,
            Err(e) => {
                log::warn!("Failed to parse EDIT chunk in bundled effect: {e}");
                None
            }
        }
    } else {
        None
    };
    effect::parse_effect(chunks.bin, config, target_location)
}
