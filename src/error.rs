//! Parser errors
//!
//! This module provides error types and utilities for parser error handling.
#[cfg(feature = "response-error")]
use http_core::response::{Response, ResponseData, StatusCode};

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
    /// The request exceeds the specied max size. 
    /// 
    /// `TooLong` may occur in these scenarios:
    /// 
    /// - The request line target exceeds the specified max size.
    /// - The header exceeds the specified max size. 
    /// 
    /// Additional scenarios may be added in future versions of this crate.
    /// 
    /// This variant contains an associated `u16` representing the recommended 
    /// HTTP status code for the error. Possible status code values:
    /// 
    /// - `414` Uri Too Long
    /// - `431` Request Header Fields Too Large
    /// 
    /// The caller is responsible for using this status code when constructing
    /// the HTTP response.
    TooLong(u16),
    /// The message body exceeded the max sized specified.
    /// 
    /// The size of body includes `CRLF` too. 
    /// 
    /// The caller may use the associated `usize` value as the response 
    /// body of an HTTP response with a `413 Content Too Larger` status code.
    /// 
    /// See more about max size configuration [`ParserConfig`].
    ContentTooLarge(usize),
    /// The HTTP request contains more items than the server allows.
    ///
    /// `TooManyItems` may occur in the following scenarios:
    ///
    /// - The request line does not contain exactly three components.
    /// - The number of header fields exceeds the configured limit (100 by default).
    ///
    /// Additional scenarios may be added in future versions of this crate.
    ///
    /// This variant contains an associated `u16` representing the recommended
    /// HTTP status code for the error.
    ///
    /// Possible status codes include:
    ///
    /// - 400 Bad Request
    /// - 431 Request Header Fields Too Large
    ///
    /// The caller is responsible for using this status code when constructing
    /// the HTTP response.
    TooManyItems(u16),
}

#[cfg(feature = "response-error")]
impl ParserErr {
    pub fn new_response_err(&self) -> Response {
        use std::io::ErrorKind;
        use StatusCode::{
            RequestTimeout, 
            BadRequest,
            ContentTooLarge,
            NotImplemented,
            RequestHeaderFieldsTooLarge,
            UriTooLong,
            HttpVersionNotSupported,
        };
        
        let mut response = Response::new();

        match self {
            Self::Io(err) => match err.kind() {
                ErrorKind::WouldBlock => response.set_status(RequestTimeout),
                ErrorKind::TimedOut => response.set_status(RequestTimeout),
                _ => response.set_status(BadRequest),
            },
            Self::TooLong(code) => {
                match code {
                    414 => response.set_status(UriTooLong),
                    431 => response.set_status(RequestHeaderFieldsTooLarge),
                    _ => response.set_status(BadRequest),
                }
            },
            Self::TooManyItems(code) => {
                match code {
                    431 => response.set_status(RequestHeaderFieldsTooLarge),
                    _ => response.set_status(BadRequest),
                }
            },
            Self::InvalidTransferEncoding(_) => response.set_status(NotImplemented),
            Self::UnsupportedHttpVersion(_) => response.set_status(HttpVersionNotSupported),
            Self::ConflictingBodyFraming => response.set_status(BadRequest),
            Self::ContentTooLarge(_) => response.set_status(ContentTooLarge),
            Self::InvalidInput(_) => response.set_status(BadRequest),   
        }

        response
    }
}

impl std::error::Error for ParserErr {}
impl std::fmt::Display for ParserErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserErr::Io(m) => write!(f, "{m}"),
            ParserErr::InvalidInput(s) => write!(f, "Unexpected input: {s}"),
            Self::UnsupportedHttpVersion(version) => write!(f, "{version}"),
            Self::TooManyItems(code) => {
                write!(f, "The request contains more items than specified.\
                    Recommended HTTP status code: {code}"
                )
            },
            Self::TooLong(code) => {
                write!(f, "The request exceeds the specified max size.\
                    Recommended HTTP status code: {code}")
            },
            Self::InvalidTransferEncoding(m) => {
                write!(f, "The HTTP request contains an unsupported\
                    or invalid coding: {m}"
                )
            },
            Self::ConflictingBodyFraming => {
                write!(f, "The HTTP request contains both\
                    Content-Length and Transfer-Encoding\
                    headers"
                )
            },
            Self::ContentTooLarge(i) => {
                write!(f, "The message body exceeded the max size: {i}")
            }
        }
    }
}

impl From<std::io::Error> for ParserErr {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
