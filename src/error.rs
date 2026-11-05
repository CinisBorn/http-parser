//! Parser errors
//!
//! This module provides error types and utilities for parser error handling.

/// The parser errors
#[derive(Debug)]
pub enum ParserErr {
    ///  A wrapper around of `std::io::Error`.
    Io(std::io::Error),
    /// An invalid HTTP request. 
    /// 
    /// This variant indicates that the request contains malformed syntax, 
    /// unsupported character encoding, or other invalid data.
    /// 
    /// The associated `String` contains more information about the cause of 
    /// error.
    /// 
    /// The caller may use the associated `String` as the response body
    /// of an HTTP response with a `400 Bad Request` status code.
    InvalidInput(String),
    /// The HTTP version used in the request is not supported.
    ///
    /// The caller may use the associated `String` as the response body
    /// of an HTTP response with a `505 HTTP Version Not Supported` status code.
    UnsupportedHttpVersion(String),
    /// The HTTP request contains both `Content-Length` and 
    /// `Transfer-Encoding` headers.
    /// 
    /// The caller may use the associated `String` as the response body
    /// of a HTTP response with `400 Bad Request` status code.
    ConflictingBodyFraming,
    /// The HTTP request contains an unsupported or 
    /// invalid transfer coding.
    ///
    /// The parser currently supports only the `chunked` 
    /// transfer coding.
    /// 
    /// The caller may use the associated `String` as the response body
    /// of an HTTP response with a `501 Not Implement` status code.
    InvalidTransferEncoding(String),
    /// Input exceed the specified max size. 
    TooLong,
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
            Self::TooLong => write!(f, ""),
            Self::InvalidTransferEncoding(m) => {
                write!(f, "The HTTP request contains an unsupported\
                    or invalid coding: {m}"
                )
            },
            ParserErr::ConflictingBodyFraming => {
                write!(f, "The HTTP request contains both\
                    Content-Length and Transfer-Encoding\
                    headers"
                )
            }
            
        }
    }
}

impl From<std::io::Error> for ParserErr {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
