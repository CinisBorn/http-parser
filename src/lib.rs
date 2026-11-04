//! Provides tools for HTTP parsing.
//!
//! This crate depends on *http-core* crate for core types such as
//! `Request` and `Response`. If you need more information about theses
//! types, refer to the *http-core* crate documentation.
//!
//! [`parse_req`] is used to parse bytes into useful HTTP `Response` representation.
//! ```no_run
//! # use std::io::Cursor;
//! # use std::io::{BufRead, BufReader, Read, Write};
//! # use http_parser::parse_req;
//! fn mock_tcp_stream(inner: &[u8]) -> Cursor<Vec<u8>> {
//!    let mut src = Cursor::new(inner.to_vec());
//!    src.set_position(0);
//!
//!    src
//! }
//!
//! let mut stream = mock_tcp_stream(b"GET / HTTP/1.1");
//! parse_req(&mut stream, None);
//! ```

pub mod error;
mod helpers;
mod policy;
mod request;
pub mod response;

pub use request::parse_req;

use crate::policy::{MAX_HEADER_SIZE, MAX_REQUEST_LINE_SIZE};

#[derive(Debug)]
/// Tracks the parsing process.
///
/// If [`ParserState::Complete`], a `Request` or `Response` is returned.
/// If [`ConnectionClosed`], the caller must handle it.
pub enum ParserState<T> {
    /// The request or response was parsed successfully.
    Complete(T),
    /// The client closed the connection.
    ConnectionClosed,
}

/// Configurations for the parsing process.
///
/// You can change the parsing behavior process setting configurations.
/// It includes, but it's not limited to: Policy of max size.
pub struct ParserConfig {
    req_line_max_size: usize,
    max_header_size: usize
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            req_line_max_size: MAX_REQUEST_LINE_SIZE,
            max_header_size: MAX_HEADER_SIZE
        }
    }
}

impl ParserConfig {
    /// Sets a request line max size. By default, the max size is `8kib` .
    /// See [`MAX_REQUEST_LINE_SIZE`].
    pub fn set_request_line_max_size(&mut self, value: usize) {
        self.req_line_max_size = value
    }

    /// Returns  the current request line max size.
    pub fn request_line_max_size(&self) -> usize {
        self.req_line_max_size
    }

    /// Sets the max size of request header. 
    pub fn set_header_max_size(&mut self, value: usize) {
        self.max_header_size = value
    }
    
    /// Returns the current header max size.
    pub fn header_max_size(&self) -> usize {
        self.max_header_size
    }
}
