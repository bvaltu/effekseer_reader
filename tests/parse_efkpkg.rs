//! Integration tests for `.efkpkg` package parsing.
//!
//! No real `.efkpkg` fixtures are committed to the repo — the Effekseer editor
//! is required to produce them. Instead, each test synthesizes an in-memory
//! zip that matches the real editor's output layout, using an existing
//! `.efkefc` fixture as the bundled effect.

use std::io::Write;

use effekseer_reader::error::Error;
use effekseer_reader::{EfkPkgFileType, load_efkpkg};

const LASER_EFKEFC: &[u8] = include_bytes!("test_data/Laser01.efkefc");

/// Build an MD5-style hash name. The real editor uses actual MD5, but
/// `load_efkpkg` doesn't validate the hash — only matches metafile keys to
/// zip entry names — so any unique string is fine here.
fn fake_hash_name(tag: &str, len: usize) -> String {
    format!("{tag:0>32}-{len:08X}")
}

/// Build a `.efkpkg` zip archive in memory from the given entries and metafile
/// JSON body.
fn build_pkg(entries: &[(&str, &[u8])], metafile_json: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut buf);
        let mut zip = zip::ZipWriter::new(cursor);
        let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("metafile.json", opts).unwrap();
        zip.write_all(metafile_json.as_bytes()).unwrap();

        for (name, data) in entries {
            zip.start_file(*name, opts).unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap();
    }
    buf
}

#[test]
fn loads_minimal_package_with_one_effect_and_one_texture() {
    let effect_hash = fake_hash_name("effect", LASER_EFKEFC.len());
    let texture_bytes: &[u8] = &[0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x11, 0x22, 0x33];
    let texture_hash = fake_hash_name("tex", texture_bytes.len());

    let metafile = format!(
        r#"{{
            "version": "1.80",
            "files": {{
                "{effect_hash}": {{
                    "type": "Effect",
                    "relative_path": "effects/laser.efkefc"
                }},
                "{texture_hash}": {{
                    "type": "Texture",
                    "relative_path": "textures/laser.png",
                    "dependencies": []
                }}
            }}
        }}"#
    );

    let pkg_bytes = build_pkg(
        &[
            (&effect_hash, LASER_EFKEFC),
            (&texture_hash, texture_bytes),
        ],
        &metafile,
    );

    let pkg = load_efkpkg(&pkg_bytes).expect("should parse valid package");

    assert_eq!(pkg.version, "1.80");
    assert_eq!(pkg.effect.file_type, EfkPkgFileType::Effect);
    assert_eq!(pkg.effect.relative_path, "effects/laser.efkefc");
    assert_eq!(pkg.effect.hash_name, effect_hash);
    assert_eq!(pkg.effect.data.len(), LASER_EFKEFC.len());
    assert!(pkg.extra_effects.is_empty());

    let tex = pkg.get(&texture_hash).expect("texture lookup by hash");
    assert_eq!(tex.file_type, EfkPkgFileType::Texture);
    assert_eq!(tex.data, texture_bytes);
    assert_eq!(tex.relative_path, "textures/laser.png");

    let same = pkg
        .find_by_relative_path("textures/laser.png")
        .expect("reverse-path lookup");
    assert_eq!(same.hash_name, texture_hash);

    // Bundled effect is a real `.efkefc` — make sure we can parse it through.
    let effect = pkg.parse_effect().expect("bundled effect should parse");
    assert!(effect.version > 0);
}

#[test]
fn missing_metafile_returns_typed_error() {
    // Zip archive with no metafile.json entry.
    let mut buf = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut buf);
        let mut zip = zip::ZipWriter::new(cursor);
        zip.start_file("random.bin", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(&[1, 2, 3]).unwrap();
        zip.finish().unwrap();
    }
    match load_efkpkg(&buf) {
        Err(Error::MissingMetafile) => {}
        other => panic!("expected MissingMetafile, got {other:?}"),
    }
}

#[test]
fn missing_entry_referenced_by_metafile_errors() {
    let effect_hash = fake_hash_name("effect", LASER_EFKEFC.len());
    let ghost_hash = fake_hash_name("ghost", 42);

    let metafile = format!(
        r#"{{
            "version": "1.80",
            "files": {{
                "{effect_hash}": {{
                    "type": "Effect",
                    "relative_path": "e.efkefc"
                }},
                "{ghost_hash}": {{
                    "type": "Texture",
                    "relative_path": "missing.png"
                }}
            }}
        }}"#
    );

    // Only write the effect entry; the ghost is advertised but absent.
    let pkg_bytes = build_pkg(&[(&effect_hash, LASER_EFKEFC)], &metafile);

    match load_efkpkg(&pkg_bytes) {
        Err(Error::MissingEntry(name)) => assert_eq!(name, ghost_hash),
        other => panic!("expected MissingEntry, got {other:?}"),
    }
}

#[test]
fn package_without_effect_entry_errors() {
    let tex_bytes: &[u8] = &[1, 2, 3, 4];
    let tex_hash = fake_hash_name("tex", tex_bytes.len());

    let metafile = format!(
        r#"{{
            "version": "1.80",
            "files": {{
                "{tex_hash}": {{
                    "type": "Texture",
                    "relative_path": "orphan.png"
                }}
            }}
        }}"#
    );

    let pkg_bytes = build_pkg(&[(&tex_hash, tex_bytes)], &metafile);

    match load_efkpkg(&pkg_bytes) {
        Err(Error::NoEffectInPackage) => {}
        other => panic!("expected NoEffectInPackage, got {other:?}"),
    }
}

#[test]
fn legacy_name_field_is_accepted_as_relative_path_alias() {
    let effect_hash = fake_hash_name("effect", LASER_EFKEFC.len());
    // Old pkgs used "name" instead of "relative_path" — see EfkPkg.cs:332 compat branch.
    let metafile = format!(
        r#"{{
            "version": "1.50",
            "files": {{
                "{effect_hash}": {{
                    "type": "Effect",
                    "name": "legacy/laser.efkefc"
                }}
            }}
        }}"#
    );

    let pkg_bytes = build_pkg(&[(&effect_hash, LASER_EFKEFC)], &metafile);
    let pkg = load_efkpkg(&pkg_bytes).expect("legacy metafile should parse");
    assert_eq!(pkg.effect.relative_path, "legacy/laser.efkefc");
}

#[test]
fn metafile_with_utf8_bom_parses() {
    // The real Effekseer editor writes metafile.json via .NET StreamWriter,
    // which emits a UTF-8 BOM (EF BB BF). The reader must strip it.
    let effect_hash = fake_hash_name("effect", LASER_EFKEFC.len());
    let body = format!(
        r#"{{"version":"1.70e","files":{{"{effect_hash}":{{"type":"Effect","relative_path":"Gun.efkproj"}}}}}}"#
    );
    let mut metafile_with_bom = Vec::new();
    metafile_with_bom.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    metafile_with_bom.extend_from_slice(body.as_bytes());

    let mut buf = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut buf);
        let mut zip = zip::ZipWriter::new(cursor);
        let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default();
        zip.start_file("metafile.json", opts).unwrap();
        zip.write_all(&metafile_with_bom).unwrap();
        zip.start_file(&effect_hash, opts).unwrap();
        zip.write_all(LASER_EFKEFC).unwrap();
        zip.finish().unwrap();
    }
    let pkg = load_efkpkg(&buf).expect("BOM-prefixed metafile should parse");
    assert_eq!(pkg.version, "1.70e");
    assert_eq!(pkg.effect.relative_path, "Gun.efkproj");
}

#[test]
fn multiple_effects_put_the_first_as_primary_and_the_rest_in_extras() {
    let a_hash = fake_hash_name("a", LASER_EFKEFC.len());
    let b_hash = fake_hash_name("b", LASER_EFKEFC.len());
    // HashMap iteration order in the metafile is not deterministic, but
    // whichever effect ends up primary, the other must appear in extras.
    let metafile = format!(
        r#"{{
            "version": "1.80",
            "files": {{
                "{a_hash}": {{ "type": "Effect", "relative_path": "a.efkefc" }},
                "{b_hash}": {{ "type": "Effect", "relative_path": "b.efkefc" }}
            }}
        }}"#
    );
    let pkg_bytes = build_pkg(
        &[(&a_hash, LASER_EFKEFC), (&b_hash, LASER_EFKEFC)],
        &metafile,
    );
    let pkg = load_efkpkg(&pkg_bytes).expect("multi-effect package should parse");
    assert_eq!(pkg.extra_effects.len(), 1);
    let all_hashes: std::collections::HashSet<_> = std::iter::once(pkg.effect.hash_name.clone())
        .chain(pkg.extra_effects.iter().map(|e| e.hash_name.clone()))
        .collect();
    assert!(all_hashes.contains(&a_hash));
    assert!(all_hashes.contains(&b_hash));
}
