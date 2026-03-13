//! `.efkefc` container extraction.
//!
//! The `.efkefc` format is a chunk-based binary container wrapping the
//! raw SKFE effect binary (in a `BIN_` chunk) plus optional editor metadata
//! (in an `EDIT` chunk).

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::primitives::Vector3D;

/// Result of extracting chunks from an `.efkefc` container.
pub(crate) struct EfkefcChunks<'a> {
    /// The raw SKFE effect binary (from the `BIN_` chunk).
    pub bin: &'a [u8],
    /// The raw `EDIT` chunk data (compressed editor XML), if present.
    pub edit: Option<&'a [u8]>,
}

/// Extract chunks from an `.efkefc` container (zero-copy).
///
/// If the data starts with `"SKFE"` (raw `.efk` file), returns the entire
/// input slice as the BIN data with no EDIT chunk. If it starts with `"EFKE"`,
/// extracts the `BIN_` and optional `EDIT` chunk payloads.
pub(crate) fn extract_chunks(data: &[u8]) -> Result<EfkefcChunks<'_>, Error> {
    if data.len() < 4 {
        return Err(Error::UnexpectedEof {
            position: 0,
            expected_bytes: 4,
        });
    }

    let magic = &data[..4];

    // Raw SKFE format — return entire input as BIN, no EDIT
    if magic == b"SKFE" {
        return Ok(EfkefcChunks {
            bin: data,
            edit: None,
        });
    }

    // Must be EFKE container
    if magic != b"EFKE" {
        return Err(Error::InvalidMagic {
            expected: b"EFKE",
            got: magic.to_vec(),
        });
    }

    let mut reader = BinaryReader::new(data);
    reader.skip(4)?; // magic
    let _container_version = reader.read_i32()?;

    let mut bin: Option<&[u8]> = None;
    let mut edit: Option<&[u8]> = None;

    // Chunk loop
    loop {
        if reader.remaining() < 8 {
            break;
        }

        let fourcc = reader.read_bytes(4)?;
        let size = reader.read_i32()?;

        if size < 0 {
            return Err(Error::InvalidChunk {
                message: format!(
                    "negative chunk size {} for chunk {:?} at position {}",
                    size,
                    String::from_utf8_lossy(fourcc),
                    reader.position() - 8,
                ),
            });
        }

        let size = size as usize;
        let chunk_data = reader.read_bytes(size)?;

        if fourcc == b"BIN_" && bin.is_none() {
            bin = Some(chunk_data);
        } else if fourcc == b"EDIT" && edit.is_none() {
            edit = Some(chunk_data);
        }
    }

    let bin = bin.ok_or_else(|| Error::MissingChunk {
        chunk_id: "BIN_".to_string(),
    })?;

    Ok(EfkefcChunks { bin, edit })
}

// ============================================================
// EDIT chunk parser — compressed XML with key/value dictionaries
// ============================================================

/// Editor behavior data extracted from the EDIT chunk.
#[derive(Debug, Clone, Default)]
pub(crate) struct EditorBehavior {
    /// Target location for attraction force fields (C++ `Manager::SetTargetLocation()`).
    pub target_location: Option<Vector3D>,
}

/// Parse the compressed EDIT chunk to extract editor behavior data.
///
/// The EDIT chunk uses zlib compression over a custom binary-encoded XML format:
/// 1. Zlib decompress
/// 2. Read key dictionary: i16 count, then count × (u16-len UTF-8 string, i16 index)
/// 3. Read value dictionary: same format
/// 4. Recursively read XML tree nodes
///
/// We only extract `TargetLocation` (X, Y, Z) from the `EffectBehavior` section.
pub(crate) fn parse_edit_chunk(data: &[u8]) -> Result<EditorBehavior, Error> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    // Step 1: Zlib decompress
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).map_err(|e| Error::InvalidChunk {
        message: format!("EDIT chunk zlib decompression failed: {e}"),
    })?;

    let mut reader = BinaryReader::new(&decompressed);

    // Step 2: Read key dictionary (element names)
    let key_count = reader.read_u16()? as usize;
    let mut keys: Vec<String> = vec![String::new(); key_count];
    for _ in 0..key_count {
        let str_len = reader.read_u16()? as usize;
        let str_bytes = reader.read_bytes(str_len)?;
        let name = std::str::from_utf8(str_bytes).map_err(|_| Error::Utf8DecodeError {
            position: reader.position() - str_len,
        })?;
        let index = reader.read_u16()? as usize;
        if index < keys.len() {
            keys[index] = name.to_string();
        }
    }

    // Step 3: Read value dictionary (text values)
    let value_count = reader.read_u16()? as usize;
    let mut values: Vec<String> = vec![String::new(); value_count];
    for _ in 0..value_count {
        let str_len = reader.read_u16()? as usize;
        let str_bytes = reader.read_bytes(str_len)?;
        let value = std::str::from_utf8(str_bytes).map_err(|_| Error::Utf8DecodeError {
            position: reader.position() - str_len,
        })?;
        let index = reader.read_u16()? as usize;
        if index < values.len() {
            values[index] = value.to_string();
        }
    }

    // Step 4: Walk the tree looking for TargetLocation
    let mut behavior = EditorBehavior::default();
    walk_tree_for_target_location(&mut reader, &keys, &values, &[], &mut behavior)?;

    Ok(behavior)
}

/// Recursively walk the compressed XML tree, tracking the element path.
/// When we find EffectBehavior > TargetLocation > {X,Y,Z}, extract the values.
fn walk_tree_for_target_location(
    reader: &mut BinaryReader,
    keys: &[String],
    values: &[String],
    path: &[&str],
    behavior: &mut EditorBehavior,
) -> Result<(), Error> {
    let element_count = reader.read_u16()? as usize;

    // Sanity limit
    if element_count > 100_000 {
        return Err(Error::InvalidChunk {
            message: format!("EDIT chunk element count too large: {element_count}"),
        });
    }

    for _ in 0..element_count {
        let name_key = reader.read_u16()? as usize;
        let name = if name_key < keys.len() {
            keys[name_key].as_str()
        } else {
            ""
        };

        // Read value (bool as i32, then optional i16 value key)
        let has_value = reader.read_i32()? != 0;
        let value_str = if has_value {
            let value_key = reader.read_u16()? as usize;
            if value_key < values.len() {
                Some(values[value_key].as_str())
            } else {
                None
            }
        } else {
            None
        };

        // Check if this is a TargetLocation child (X, Y, or Z).
        // XML path is: EffekseerProject > Behavior > TargetLocation > {X, Y, Z}
        if path.len() >= 1 && path[path.len() - 1] == "TargetLocation" {
            if let Some(val) = value_str {
                if let Ok(f) = val.parse::<f32>() {
                    let tl = behavior.target_location.get_or_insert(Vector3D {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    });
                    match name {
                        "X" => tl.x = f,
                        "Y" => tl.y = f,
                        "Z" => tl.z = f,
                        _ => {}
                    }
                }
            }
        }

        // Read children
        let has_children = reader.read_i32()? != 0;
        if has_children {
            let mut new_path: Vec<&str> = path.to_vec();
            new_path.push(name);
            walk_tree_for_target_location(reader, keys, values, &new_path, behavior)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal EFKE container with given chunks.
    fn build_efkefc(container_version: i32, chunks: &[(&[u8; 4], &[u8])]) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(b"EFKE");
        data.extend_from_slice(&container_version.to_le_bytes());
        for (fourcc, payload) in chunks {
            data.extend_from_slice(*fourcc);
            data.extend_from_slice(&(payload.len() as i32).to_le_bytes());
            data.extend_from_slice(payload);
        }
        data
    }

    #[test]
    fn test_extract_chunks() {
        let bin_payload = b"SKFE_fake_data";
        let container = build_efkefc(1, &[(b"INFO", b"info"), (b"BIN_", bin_payload)]);
        let result = extract_chunks(&container).unwrap();
        assert_eq!(result.bin, bin_payload);
        assert!(result.edit.is_none());
    }

    #[test]
    fn test_extract_chunks_with_edit() {
        let bin_payload = b"SKFE_fake_data";
        let edit_payload = b"edit_data";
        let container = build_efkefc(
            1,
            &[
                (b"INFO", b"info"),
                (b"BIN_", bin_payload),
                (b"EDIT", edit_payload),
            ],
        );
        let chunks = extract_chunks(&container).unwrap();
        assert_eq!(chunks.bin, bin_payload);
        assert_eq!(chunks.edit, Some(edit_payload.as_slice()));
    }

    #[test]
    fn test_skfe_passthrough() {
        let data = b"SKFEsomedata";
        let result = extract_chunks(data).unwrap();
        assert_eq!(result.bin, &data[..]);
        assert!(result.edit.is_none());
    }

    #[test]
    fn test_skfe_passthrough_no_edit() {
        let data = b"SKFEsomedata";
        let chunks = extract_chunks(data).unwrap();
        assert_eq!(chunks.bin, &data[..]);
        assert!(chunks.edit.is_none());
    }

    #[test]
    fn test_invalid_magic() {
        let data = b"BADMsomedata";
        let result = extract_chunks(data);
        assert!(matches!(result, Err(Error::InvalidMagic { .. })));
    }

    #[test]
    fn test_missing_bin_chunk() {
        let container = build_efkefc(1, &[(b"INFO", b"info"), (b"EDIT", b"edit")]);
        let result = extract_chunks(&container);
        assert!(matches!(result, Err(Error::MissingChunk { .. })));
    }

    #[test]
    fn test_empty_bin_chunk() {
        let container = build_efkefc(1, &[(b"BIN_", b"")]);
        let result = extract_chunks(&container).unwrap();
        assert!(result.bin.is_empty());
    }

    #[test]
    fn test_zero_size_chunk_skipped() {
        let bin_payload = b"SKFE_data";
        let container = build_efkefc(1, &[(b"INFO", b""), (b"BIN_", bin_payload)]);
        let result = extract_chunks(&container).unwrap();
        assert_eq!(result.bin, bin_payload);
    }

    #[test]
    fn test_negative_chunk_size() {
        let mut data = Vec::new();
        data.extend_from_slice(b"EFKE");
        data.extend_from_slice(&1i32.to_le_bytes());
        data.extend_from_slice(b"BIN_");
        data.extend_from_slice(&(-1i32).to_le_bytes());
        let result = extract_chunks(&data);
        assert!(matches!(result, Err(Error::InvalidChunk { .. })));
    }

    #[test]
    fn test_truncated_header() {
        let data = b"EF";
        let result = extract_chunks(data);
        assert!(matches!(result, Err(Error::UnexpectedEof { .. })));
    }

    #[test]
    fn test_parse_edit_chunk_synthetic() {
        // Build a synthetic compressed EDIT chunk with TargetLocation data.
        // XML structure: <EffekseerProject><Behavior><TargetLocation><X>1.5</X><Y>15</Y><Z>-3</Z></TargetLocation></Behavior></EffekseerProject>
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut buf = Vec::new();

        // Key dictionary: 6 keys
        let key_names = [
            "EffekseerProject",
            "Behavior",
            "TargetLocation",
            "X",
            "Y",
            "Z",
        ];
        buf.extend_from_slice(&(key_names.len() as u16).to_le_bytes());
        for (i, name) in key_names.iter().enumerate() {
            let bytes = name.as_bytes();
            buf.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
            buf.extend_from_slice(bytes);
            buf.extend_from_slice(&(i as u16).to_le_bytes());
        }

        // Value dictionary: 3 values
        let value_strs = ["1.5", "15", "-3"];
        buf.extend_from_slice(&(value_strs.len() as u16).to_le_bytes());
        for (i, val) in value_strs.iter().enumerate() {
            let bytes = val.as_bytes();
            buf.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
            buf.extend_from_slice(bytes);
            buf.extend_from_slice(&(i as u16).to_le_bytes());
        }

        // Tree: EffekseerProject (1 element at top level)
        buf.extend_from_slice(&1u16.to_le_bytes()); // 1 top-level element
        // EffekseerProject element
        buf.extend_from_slice(&0u16.to_le_bytes()); // name_key = 0 ("EffekseerProject")
        buf.extend_from_slice(&0i32.to_le_bytes()); // has_value = false
        buf.extend_from_slice(&1i32.to_le_bytes()); // has_children = true
        {
            // EffekseerProject children: 1 element (Behavior)
            buf.extend_from_slice(&1u16.to_le_bytes());
            buf.extend_from_slice(&1u16.to_le_bytes()); // name_key = 1 ("Behavior")
            buf.extend_from_slice(&0i32.to_le_bytes()); // has_value = false
            buf.extend_from_slice(&1i32.to_le_bytes()); // has_children = true
            {
                // Behavior children: 1 element (TargetLocation)
                buf.extend_from_slice(&1u16.to_le_bytes());
                buf.extend_from_slice(&2u16.to_le_bytes()); // name_key = 2 ("TargetLocation")
                buf.extend_from_slice(&0i32.to_le_bytes()); // has_value = false
                buf.extend_from_slice(&1i32.to_le_bytes()); // has_children = true
                {
                    // TargetLocation children: 3 elements (X, Y, Z)
                    buf.extend_from_slice(&3u16.to_le_bytes());

                    // X
                    buf.extend_from_slice(&3u16.to_le_bytes()); // name_key = 3 ("X")
                    buf.extend_from_slice(&1i32.to_le_bytes()); // has_value = true
                    buf.extend_from_slice(&0u16.to_le_bytes()); // value_key = 0 ("1.5")
                    buf.extend_from_slice(&0i32.to_le_bytes()); // has_children = false

                    // Y
                    buf.extend_from_slice(&4u16.to_le_bytes()); // name_key = 4 ("Y")
                    buf.extend_from_slice(&1i32.to_le_bytes()); // has_value = true
                    buf.extend_from_slice(&1u16.to_le_bytes()); // value_key = 1 ("15")
                    buf.extend_from_slice(&0i32.to_le_bytes()); // has_children = false

                    // Z
                    buf.extend_from_slice(&5u16.to_le_bytes()); // name_key = 5 ("Z")
                    buf.extend_from_slice(&1i32.to_le_bytes()); // has_value = true
                    buf.extend_from_slice(&2u16.to_le_bytes()); // value_key = 2 ("-3")
                    buf.extend_from_slice(&0i32.to_le_bytes()); // has_children = false
                }
            }
        }

        // Zlib compress
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&buf).unwrap();
        let compressed = encoder.finish().unwrap();

        let behavior = parse_edit_chunk(&compressed).unwrap();
        let tl = behavior.target_location.expect("should have target_location");
        assert!((tl.x - 1.5).abs() < 0.001);
        assert!((tl.y - 15.0).abs() < 0.001);
        assert!((tl.z - (-3.0)).abs() < 0.001);
    }
}
