//! Provides tools for client request handling

mod body;
mod header_section;

use http_core::request::Request;
use std::io::{BufReader, Read, Write};

use crate::ParserConfig;
use crate::ParserState;
use crate::error::ParserErr;
use crate::request::body::decode_chunked_body;

use body::read_body;
use body::check;

use header_section::parse_headers;
use header_section::parse_req_line;

/// Parses a incoming request.
///
/// # Errors
///
/// Returns a [`ParserErr`] if it's not possible to parse the request.
pub fn parse_req<T>(
    src: &mut T, 
    config: Option<&ParserConfig>
) -> Result<ParserState<Request>, ParserErr>
where
    T: Read + Write,
{
    let mut reader = BufReader::new(src);
    let config = match config {
        Some(conf) => conf,
        None => &ParserConfig::default()
    };
    
    let ParserState::Complete(start_line) = parse_req_line(&mut reader, config)? else {
        return Ok(ParserState::ConnectionClosed);
    };
    let ParserState::Complete(headers) = parse_headers(&mut reader, config)? else {
        return Ok(ParserState::ConnectionClosed);
    };

    let method = check::body_framing_method(headers.as_ref());

    match method {
        Ok(body_method) => match body_method {
            check::BodyProcessMethod::ContentLength => {
                let body = match read_body(&mut reader, headers.as_ref(), config)? {
                    ParserState::Complete(value) => value,
                    ParserState::ConnectionClosed => return Ok(ParserState::ConnectionClosed)
                };
                
                Ok(ParserState::Complete(Request::new(
                    start_line, headers, body,
                )))
            },
            check::BodyProcessMethod::TransferCoding => {
                let body = match decode_chunked_body(&mut reader, headers.as_ref(), config)? {
                    ParserState::Complete(value) => value,
                    ParserState::ConnectionClosed => return Ok(ParserState::ConnectionClosed)
                };

                Ok(ParserState::Complete(Request::new(
                    start_line, headers, body,
                )))
            },
            check::BodyProcessMethod::None => {
                Ok(ParserState::Complete(Request::new(
                    start_line, headers, None,
                )))
            }
        },
        Err(e) => Err(e),
    }
}
