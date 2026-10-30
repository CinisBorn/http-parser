//! Parser errors
//!
//! This module provides error types and utilities for parser error handling.

/// The parser errors
#[derive(Debug)]
pub enum ParserErr {
    ///  A wrapper around of `std::io::Error`.
    Io(std::io::Error),
    /// Unexpected input was found.
    InvalidInput(String),
    /// The client made a request with an unsupported HTTP version.
    UnsupportedHttpVersion(String),
    /// The request contains more items than expected.
    TooManyItems,
}

impl std::error::Error for ParserErr {}
impl std::fmt::Display for ParserErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserErr::Io(m) => write!(f, "{m}"),
            ParserErr::InvalidInput(s) => write!(f, "Unexpected input: {s}"),
            Self::UnsupportedHttpVersion(version) => write!(f, "{version}"),
            Self::TooManyItems => write!(f, "The request contains more items than expected"),
        }
    }
}

impl From<std::io::Error> for ParserErr {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
