//! Contains function for parsing headers and request line sections.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};

use http_core::Method;
use http_core::request::{HttpVersion, RequestLine};

use crate::error::ParserErr;
use crate::helpers::find_line;
use crate::ParserState;

pub(crate) fn parse_headers<T>(
    src: &mut BufReader<T>,
) -> Result<ParserState<Option<HashMap<String, String>>>, ParserErr>
where
    T: Read + Write,
{
    let mut headers = HashMap::new();

    loop {
        let buf = src.fill_buf().map_err(|e| ParserErr::Io(e))?;

        if buf.is_empty() {
            return Ok(ParserState::ConnectionClosed)
        }

        if let Some(pos) = find_line(buf) {
            let line = &buf[..pos].to_owned();
            src.consume(line.len() + 2);

            if line.is_empty() {
                if headers.is_empty() {
                    break Ok(ParserState::Complete(None));
                } else {
                    break Ok(ParserState::Complete(Some(headers)));
                }
            }

            let Some(colon_position) = line.iter().position(|byte| *byte == b':') else {
                return Err(ParserErr::InvalidInput(String::from("No Colon Found")));
            };

            let field = &line[..colon_position];
            let value = &line[colon_position + 2..];

            let Ok(field) = std::str::from_utf8(field) else {
                return Err(ParserErr::InvalidInput(
                    String::from_utf8(field.to_vec()).unwrap(),
                ));
            };

            let Ok(value) = std::str::from_utf8(value) else {
                return Err(ParserErr::InvalidInput(
                    String::from_utf8(value.to_vec()).unwrap(),
                ));
            };

            headers.insert(field.to_owned(), value.to_owned());
        }
    }
}

pub(crate) fn parse_req_line<T>(src: &mut BufReader<T>) -> Result<ParserState<RequestLine>, ParserErr> 
where 
    T: Read
{
    loop {
        let buf = src.fill_buf().map_err(|e| ParserErr::Io(e))?;

        if buf.is_empty() {
            return Ok(ParserState::ConnectionClosed);
        }

        if let Some(pos) = find_line(buf) {
            let line = buf[..pos].to_owned();
            src.consume(pos + 2);

            let mut items = line.split(|byte| *byte == b' ');

            let met = items
                .next()
                .ok_or(ParserErr::InvalidInput("Method not found".into()))?;
            let tar = items
                .next()
                .ok_or(ParserErr::InvalidInput("Target not Found".into()))?;
            let ver = items
                .next()
                .ok_or(ParserErr::InvalidInput("Version not Found".into()))?;

            if items.next().is_some() {
                return Err(ParserErr::TooManyItems);
            }

            let method = std::str::from_utf8(met)
                .map_err(|_| ParserErr::InvalidInput("Invalid HTTP method encoding".into()))?;

            let target = std::str::from_utf8(tar)
                .map_err(|_| ParserErr::InvalidInput("Invalid target encoding".into()))?;

            let version = std::str::from_utf8(ver)
                .map_err(|_| ParserErr::InvalidInput("Invalid HTTP version encoding".into()))?;

            let met = Method::map_method(method);
            let ver = HttpVersion::map(version);

            if let HttpVersion::Unsupported(v) = ver {
                return Err(ParserErr::UnsupportedHttpVersion(v.clone()));
            }

            let req_start_line = RequestLine::new(ver, met, target.to_owned());

            break Ok(ParserState::Complete(req_start_line));
        }
    }
}

use std::io::Cursor;

#[allow(dead_code)] // clippy issues 
fn mock_reader(inner: &[u8]) -> BufReader<Cursor<Vec<u8>>> {
    let mut src = Cursor::new(inner.to_vec());
    src.set_position(0);

    BufReader::new(src)
}

#[cfg(test)]
mod request_line {
    use super::*;

    #[test]
    fn parse_request_line() {
        let mut src = mock_reader(b"GET / HTTP/1.1\r\n\r\n");
        parse_req_line(&mut src).unwrap();
    }

    #[test]
    fn parsed_content_is_correct() {
        let mut src = mock_reader(b"GET / HTTP/1.1\r\n\r\n");
        let res = match parse_req_line(&mut src).unwrap() {
            ParserState::Complete(res) => res,
            ParserState::ConnectionClosed => panic!("Connection Closed")
        };

        match res.method() {
            Method::GET => assert!(true),
            _ => assert!(false),
        }

        match res.version() {
            HttpVersion::V1 => assert!(true),
            _ => assert!(false),
        }

        assert_eq!(res.target(), "/");
    }

    #[test]
    fn unsupported_http_version_fail() -> Result<(), ()> {
        let mut src = mock_reader(b"GET / HTTP/2.0\r\n\r\n");
        let res = parse_req_line(&mut src).err().unwrap();

        match res {
            ParserErr::UnsupportedHttpVersion(_) => return Ok(()),
            _ => return Err(()),
        }
    }

    #[test]
    fn invalid_input_fail() -> Result<(), ()> {
        let mut src = mock_reader(b"GET HTTP/2.0\r\n\r\n");
        let res = parse_req_line(&mut src).err().unwrap();

        match res {
            ParserErr::InvalidInput(_) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    fn too_many_items_fail() -> Result<(), ()> {
        let mut src = mock_reader(b"GET / HTTP/2.0 abc\r\n\r\n");
        let res = parse_req_line(&mut src).err().unwrap();

        match res {
            ParserErr::TooManyItems => Ok(()),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod headers_parsing {
    use super::*;

    #[test]
    fn header_is_parsed() {
        let mut src = mock_reader(
            b"Host: 127.0.0.1\r\n\
            \r\n",
        );

        parse_headers(&mut src).unwrap();
    }

    #[test]
    fn header_content_is_correct() {
        let mut src = mock_reader(
            b"Host: 127.0.0.1\r\n\
            \r\n",
        );
        let res = match parse_headers(&mut src).unwrap() {
            ParserState::Complete(res) => res.unwrap(),
            ParserState::ConnectionClosed => panic!("Connection was closed") 
        };
        
        assert_eq!(res.get("Host").unwrap(), "127.0.0.1");
    }

    #[test]
    fn request_without_headers_pass() -> Result<(), ()> {
        let mut src = mock_reader(b"\r\n");
        let res = match parse_headers(&mut src).unwrap() {
            ParserState::Complete(res) => res,
            ParserState::ConnectionClosed => panic!("Connection was closed") 
        };

        match res {
            Some(_) => Err(()),
            None => Ok(()),
        }
    }

    #[test]
    fn bad_formatted_headers_fail() -> Result<(), ()> {
        let mut src = mock_reader(b"host 127.0.0.1\r\n\r\n");
        let res = parse_headers(&mut src).unwrap_err();

        match res {
            ParserErr::InvalidInput(_) => Ok(()),
            _ => Err(()),
        }
    }
}
