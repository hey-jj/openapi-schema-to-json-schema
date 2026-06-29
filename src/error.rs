//! Error types returned by the converters.

use std::fmt;

/// Errors raised during conversion.
///
/// The conversion functions are fallible. Three conditions stop a conversion:
/// an input `type` value that is not a valid JSON Schema type, a root schema
/// that is neither an object nor an array, and a parameter with neither a
/// `schema` nor a `content` member while strict mode is on.
///
/// Match on the variant to branch on the cause. Each variant carries the full
/// message, which is also what [`Display`](std::fmt::Display) prints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A node carried a `type` value outside the draft-04 type set while strict
    /// mode was on. The string is the full message, including the offending
    /// value rendered as JSON.
    InvalidType(String),
    /// A root schema was a scalar or null, or a parameter had no `schema` and no
    /// `content` while strict mode was on.
    InvalidInput(String),
}

impl Error {
    /// The error message.
    pub fn message(&self) -> &str {
        match self {
            Error::InvalidType(m) | Error::InvalidInput(m) => m,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for Error {}
