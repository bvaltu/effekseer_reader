//! Error types for the effekseer_reader crate.

/// Errors that can occur during parsing of Effekseer binary files.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid magic bytes at the start of a file or chunk.
    #[error("invalid magic bytes: expected {expected:?}, got {got:?}")]
    InvalidMagic {
        /// The expected magic byte sequence.
        expected: &'static [u8],
        /// The actual bytes found.
        got: Vec<u8>,
    },

    /// The file version is not supported by this parser.
    #[error("unsupported version: {version}")]
    UnsupportedVersion {
        /// The version number read from the file.
        version: i32,
    },

    /// Unexpected end of file while reading data.
    #[error("unexpected end of file at position {position}, needed {expected_bytes} more bytes")]
    UnexpectedEof {
        /// The byte position where the read was attempted.
        position: usize,
        /// The number of additional bytes needed.
        expected_bytes: usize,
    },

    /// An enum field contained an unrecognized discriminant value.
    #[error("invalid enum value for {field}: {value}")]
    InvalidEnumValue {
        /// The name of the field being parsed.
        field: &'static str,
        /// The raw integer value that was not recognized.
        value: i32,
    },

    /// A UTF-16 encoded string could not be decoded.
    #[error("UTF-16 decode error at position {position}")]
    Utf16DecodeError {
        /// The byte position where decoding failed.
        position: usize,
    },

    /// A UTF-8 encoded string could not be decoded.
    #[error("UTF-8 decode error at position {position}")]
    Utf8DecodeError {
        /// The byte position where decoding failed.
        position: usize,
    },

    /// A resource count exceeded the configured limit.
    #[error("resource limit exceeded for {field}: {count} exceeds max {max}")]
    ResourceLimitExceeded {
        /// The name of the resource being counted.
        field: &'static str,
        /// The actual count encountered.
        count: usize,
        /// The configured maximum.
        max: usize,
    },

    /// A chunk in the container file is invalid.
    #[error("invalid chunk: {message}")]
    InvalidChunk {
        /// Description of the chunk error.
        message: String,
    },

    /// A required chunk was not found in the container file.
    #[error("missing required chunk: {chunk_id}")]
    MissingChunk {
        /// The four-byte chunk identifier that was expected.
        chunk_id: String,
    },

    /// An error occurred while reading the `.efkpkg` zip archive.
    #[error("zip error: {0}")]
    ZipError(String),

    /// An error occurred while parsing `metafile.json` in a `.efkpkg`.
    #[error("metafile JSON error: {0}")]
    JsonError(String),

    /// The `.efkpkg` did not contain the required `metafile.json` entry.
    #[error("missing metafile.json in .efkpkg")]
    MissingMetafile,

    /// A file listed in `metafile.json` was not present in the zip archive.
    #[error("missing entry in .efkpkg: {0}")]
    MissingEntry(String),

    /// The `.efkpkg` contained no file of type `Effect`.
    #[error("no effect found in .efkpkg")]
    NoEffectInPackage,
}
