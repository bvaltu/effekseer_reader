//! `.efkefc` container extraction.
//!
//! The `.efkefc` format is a chunk-based binary container wrapping the
//! raw SKFE effect binary (in a `BIN_` chunk) plus optional editor metadata.

use crate::error::Error;
use crate::reader::BinaryReader;

/// Extract the SKFE effect binary from an `.efkefc` container (zero-copy).
///
/// If the data starts with `"SKFE"` (raw `.efk` file), returns the entire
/// input slice as-is. If it starts with `"EFKE"`, extracts the first `BIN_`
/// chunk payload. Returns an error for any other magic or if no `BIN_` chunk
/// is found.
pub(crate) fn extract_bin_chunk(data: &[u8]) -> Result<&[u8], Error> {
    if data.len() < 4 {
        return Err(Error::UnexpectedEof {
            position: 0,
            expected_bytes: 4,
        });
    }

    let magic = &data[..4];

    // Raw SKFE format — return entire input
    if magic == b"SKFE" {
        return Ok(data);
    }

    // Must be EFKE container
    if magic != b"EFKE" {
        return Err(Error::InvalidMagic {
            expected: b"EFKE",
            got: magic.to_vec(),
        });
    }

    let mut reader = BinaryReader::new(data);
    // Skip the 4-byte magic we already validated
    reader.skip(4)?;
    // Read container version (stored but not validated)
    let _container_version = reader.read_i32()?;

    // Chunk loop
    loop {
        if reader.remaining() < 8 {
            break;
        }

        let fourcc = reader.read_bytes(4)?;
        let size = reader.read_i32()?;

        // Negative chunk size is invalid
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

        if fourcc == b"BIN_" {
            let bin_data = reader.read_bytes(size)?;
            return Ok(bin_data);
        }

        // Skip non-BIN_ chunks
        reader.skip(size)?;
    }

    Err(Error::MissingChunk {
        chunk_id: "BIN_".to_string(),
    })
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
    fn test_extract_bin_chunk() {
        let bin_payload = b"SKFE_fake_data";
        let container = build_efkefc(1, &[(b"INFO", b"info"), (b"BIN_", bin_payload)]);
        let result = extract_bin_chunk(&container).unwrap();
        assert_eq!(result, bin_payload);
    }

    #[test]
    fn test_skfe_passthrough() {
        let data = b"SKFEsomedata";
        let result = extract_bin_chunk(data).unwrap();
        assert_eq!(result, &data[..]);
    }

    #[test]
    fn test_invalid_magic() {
        let data = b"BADMsomedata";
        let result = extract_bin_chunk(data);
        assert!(matches!(result, Err(Error::InvalidMagic { .. })));
    }

    #[test]
    fn test_missing_bin_chunk() {
        let container = build_efkefc(1, &[(b"INFO", b"info"), (b"EDIT", b"edit")]);
        let result = extract_bin_chunk(&container);
        assert!(matches!(result, Err(Error::MissingChunk { .. })));
    }

    #[test]
    fn test_empty_bin_chunk() {
        let container = build_efkefc(1, &[(b"BIN_", b"")]);
        let result = extract_bin_chunk(&container).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_zero_size_chunk_skipped() {
        let bin_payload = b"SKFE_data";
        let container = build_efkefc(1, &[(b"INFO", b""), (b"BIN_", bin_payload)]);
        let result = extract_bin_chunk(&container).unwrap();
        assert_eq!(result, bin_payload);
    }

    #[test]
    fn test_negative_chunk_size() {
        let mut data = Vec::new();
        data.extend_from_slice(b"EFKE");
        data.extend_from_slice(&1i32.to_le_bytes());
        data.extend_from_slice(b"BIN_");
        data.extend_from_slice(&(-1i32).to_le_bytes());
        let result = extract_bin_chunk(&data);
        assert!(matches!(result, Err(Error::InvalidChunk { .. })));
    }

    #[test]
    fn test_truncated_header() {
        let data = b"EF";
        let result = extract_bin_chunk(data);
        assert!(matches!(result, Err(Error::UnexpectedEof { .. })));
    }
}
