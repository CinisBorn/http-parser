//! Parser errors
//!
//! This module provides error types and utilities for parser error handling.

/// The parser errors
#[derive(Debug)]
pub enum ParserErr {
    ///  A wrapper around of `std::io::Error`.
    Io(std::io::Error),
    /// The connection was closed.
    ///
    /// This error occurs when the client close the connection when the server is waiting
    /// for new incoming data.
    ConnClosed,
    /// It's a temporary measure why the understading of "unknown method" isn't well defined.
    /// It's very common costumized methods or fields in HTTP requests because HTTP is extensible by design.
    UnknownMethod(String),
    /// Unexpected input was found.
    InvalidInput(String),
    /// The parsing process was ignored.
    ///
    /// The main purpose of this variant is to signal a process was skipped.
    Ignored,
    /// The client made a request with an unsupported HTTP version.
    UnsupportedHttpVersion(String),
    /// The request contains more items than expected.
    TooManyItems,
}

impl std::error::Error for ParserErr {}
impl std::fmt::Display for ParserErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserErr::UnknownMethod(m) => write!(f, "{m}"),
            ParserErr::ConnClosed => write!(f, "Connection was closed"),
            ParserErr::Io(m) => write!(f, "{m}"),
            ParserErr::InvalidInput(s) => write!(f, "Unexpected input: {s}"),
            ParserErr::Ignored => write!(f, ""),
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
