use std::fmt;

#[derive(Debug)]
#[non_exhaustive]
/// Errors that can occur during MessagePack encoding/decoding operations
pub enum Error {
    /// Encountered a MessagePack type that doesn't match the expected type
    UnexpectedType,
    /// Not enough data available to decode a complete MessagePack value
    InsufficientData,
    /// The MessagePack data is malformed or corrupted
    InvalidData,
    /// Encountered a field name that isn't recognized
    UnknownField(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnexpectedType => write!(f, "Unexpected MessagePack type"),
            Error::InsufficientData => write!(f, "Insufficient data to decode"),
            Error::InvalidData => write!(f, "Invalid MessagePack data"),
            Error::UnknownField(field) => write!(f, "Unknown field: {}", field),
        }
    }
}

impl std::error::Error for Error {}
