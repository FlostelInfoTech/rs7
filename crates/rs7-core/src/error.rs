//! Error types for HL7 message processing

use thiserror::Error;

/// Result type alias for rs7 operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the rs7 library
#[derive(Error, Debug)]
pub enum Error {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Invalid delimiter configuration: {0}")]
    InvalidDelimiters(String),

    #[error("Invalid encoding: {0}")]
    InvalidEncoding(String),

    #[error("Invalid segment: {0}")]
    InvalidSegment(String),

    #[error("Invalid field access: {0}")]
    InvalidFieldAccess(String),

    #[error("Missing required field: {0}")]
    MissingRequiredField(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(String),

    #[error("Message type error: {0}")]
    MessageType(String),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Decoding error: {0}")]
    Decoding(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("MLLP error: {0}")]
    Mllp(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Terser path error: {0}")]
    TerserPath(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl Error {
    /// Create a parse error
    pub fn parse<S: Into<String>>(msg: S) -> Self {
        Error::Parse(msg.into())
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Error::Validation(msg.into())
    }

    /// Create a terser path error
    pub fn terser_path<S: Into<String>>(msg: S) -> Self {
        Error::TerserPath(msg.into())
    }
}
