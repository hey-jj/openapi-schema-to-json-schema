//! Error types returned by the converters.

use std::fmt;

/// Errors raised during conversion.
///
/// The conversion functions are fallible. Two conditions stop a conversion:
/// an input `type` value that is not a valid JSON Schema type, and a parameter
/// with neither a `schema` nor a `content` member while strict mode is on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A node carried a `type` value outside the draft-04 type set while strict
    /// mode was on. The string is the full message, including the offending
    /// value rendered as JSON.
    InvalidType(String),
    /// A parameter or response had no `schema` and no `content` while strict
    /// mode was on.
    InvalidInput(String),
}

impl Error {
    /// The error name, matching the class name in the source library.
    ///
    /// Returns `"InvalidTypeError"` or `"InvalidInputError"`.
    pub fn name(&self) -> &'static str {
        match self {
            Error::InvalidType(_) => "InvalidTypeError",
            Error::InvalidInput(_) => "InvalidInputError",
        }
    }

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
