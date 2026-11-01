//! Provides tools for client request handling

mod body;
mod header_section;

use http_core::request::Request;
use std::io::{BufReader, Read, Write};

use crate::error::ParserErr;
use crate::ParserState;
use crate::request::body::decode_chunked_body;
use crate::request::header_section::validate_body_header;

use body::read_body;
use header_section::parse_headers;
use header_section::parse_req_line;

/// Parses a incoming request.
///
/// # Errors
///
/// Returns a [`ParserErr`] if it's not possible to parse the request.
pub fn parse_req<T>(src: &mut T) -> Result<ParserState<Request>, ParserErr>
where
    T: Read + Write,
{
    let mut reader = BufReader::new(src);

    let ParserState::Complete(start_line) = parse_req_line(&mut reader)? else {
        return Ok(ParserState::ConnectionClosed);
    };
    let ParserState::Complete(headers) = parse_headers(&mut reader)? else {
        return Ok(ParserState::ConnectionClosed);
    };

    let method = validate_body_header(&headers);

    match method {
        Ok(body_method) => {
            match body_method {
                header_section::BodyProcessMethod::ContentLength => {
                    let ParserState::Complete(body) = read_body(&mut reader, headers.as_ref())? else {
                        return Ok(ParserState::ConnectionClosed);
                    };
                    
                    return Ok(ParserState::Complete(Request::new(start_line, headers, body)))
                },
                header_section::BodyProcessMethod::TransferCoding => {
                    let ParserState::Complete(body) = decode_chunked_body(&mut reader, headers.as_ref())? else {
                        return Ok(ParserState::ConnectionClosed)
                    };
                    
                    return Ok(ParserState::Complete(Request::new(start_line, headers, body)))
                },
                header_section::BodyProcessMethod::None => {
                    return Ok(ParserState::Complete(Request::new(start_line, headers, None)))
                }
            }
        },
        Err(e) => Err(e)
    }
}
