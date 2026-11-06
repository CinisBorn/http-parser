//! Contains function for parsing headers and request line sections.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};

use http_core::Method;
use http_core::request::{HttpVersion, RequestLine};

use crate::error::ParserErr;
use crate::helpers::find_line;
use crate::{ParserConfig, ParserState};

pub(crate) fn parse_headers<T>(
    src: &mut BufReader<T>,
    config: &ParserConfig
) -> Result<ParserState<Option<HashMap<String, String>>>, ParserErr>
where
    T: Read + Write,
{
    let mut headers = HashMap::new();
    let mut size: usize = 0;
    
    loop {
        let buf = src.fill_buf().map_err(|e| ParserErr::Io(e))?;
      
        if buf.is_empty() {
            return Ok(ParserState::ConnectionClosed)
        }

        if size > config.header_max_size() {
            return Err(ParserErr::TooLong(431))
        }
        
        if headers.len() >= config.number_of_headers() {
            return Err(ParserErr::TooManyItems(431))
        }
        
        if let Some(pos) = find_line(buf) {
            let line = &buf[..pos].to_owned();
            let length = line.len() + 2;
            
            size += length;
            src.consume(length);

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

            let field = std::str::from_utf8(field).map_err(|err| {
                ParserErr::InvalidInput(err.to_string())
            })?;

            let value = std::str::from_utf8(value).map_err(|err| {
                ParserErr::InvalidInput(err.to_string())
            })?;

            headers.insert(field.to_owned(), value.to_owned());
        }
    }
}

pub(crate) fn parse_req_line<T>(
    src: &mut BufReader<T>, 
    config:  &ParserConfig
) -> Result<ParserState<RequestLine>, ParserErr> 
where 
    T: Read
{
    loop {
        let buf = src.fill_buf().map_err(|e| ParserErr::Io(e))?;

        if buf.is_empty() {
            return Ok(ParserState::ConnectionClosed);
        }

        if buf.len() > config.request_line_max_size() {
            return Err(ParserErr::TooLong(414))
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
                return Err(ParserErr::TooManyItems(400));
            }

            let method = std::str::from_utf8(met).map_err(|err| {
                ParserErr::InvalidInput(err.to_string())
            })?;
            
            let target = std::str::from_utf8(tar).map_err(|err| {
                ParserErr::InvalidInput(err.to_string())
            })?;

            let version = std::str::from_utf8(ver).map_err(|err| {
                ParserErr::InvalidInput(err.to_string())
            })?;

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
        let config = ParserConfig::default();
        parse_req_line(&mut src, &config).unwrap();
    }

    #[test]
    fn parsed_content_is_correct() {
        let mut src = mock_reader(b"GET / HTTP/1.1\r\n\r\n");
        let config = ParserConfig::default();
        let res = match parse_req_line(&mut src, &config).unwrap() {
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
        let config = ParserConfig::default();
        let res = parse_req_line(&mut src, &config).err().unwrap();

        match res {
            ParserErr::UnsupportedHttpVersion(_) => return Ok(()),
            _ => return Err(()),
        }
    }

    #[test]
    fn invalid_input_fail() -> Result<(), ()> {
        let mut src = mock_reader(b"GET HTTP/2.0\r\n\r\n");
        let config = ParserConfig::default();
        let res = parse_req_line(&mut src, &config).err().unwrap();

        match res {
            ParserErr::InvalidInput(_) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    fn too_many_items_fail() -> Result<(), ()> {
        let mut src = mock_reader(b"GET / HTTP/2.0 abc\r\n\r\n");
        let config = ParserConfig::default();
        let res = parse_req_line(&mut src, &config).err().unwrap();

        match res {
            ParserErr::TooManyItems(_) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    #[should_panic]
    fn fail_if_max_size_is_exceeded() {
        let mut reader = mock_reader(b"GET /test HTTP/1.1\r\n\r\n");
        let mut config = ParserConfig::default();

        config.set_request_line_max_size(1);
        
        let _ = parse_req_line(&mut reader, &config).unwrap();
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

        parse_headers(&mut src, &ParserConfig::default()).unwrap();
    }

    #[test]
    fn header_content_is_correct() {
        let mut src = mock_reader(
            b"Host: 127.0.0.1\r\n\
            \r\n",
        );

        let res = match parse_headers(&mut src, &ParserConfig::default()).unwrap() {
            ParserState::Complete(res) => res.unwrap(),
            ParserState::ConnectionClosed => panic!("Connection was closed") 
        };
        
        assert_eq!(res.get("Host").unwrap(), "127.0.0.1");
    }

    #[test]
    fn request_without_headers_pass() -> Result<(), ()> {
        let mut src = mock_reader(b"\r\n");

        let res = match parse_headers(&mut src, &ParserConfig::default()).unwrap() {
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
        
        let res = parse_headers(&mut src, &ParserConfig::default()).unwrap_err();

        match res {
            ParserErr::InvalidInput(_) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    #[should_panic]
    fn fail_if_max_size_exceed() {
        let mut src = mock_reader(b"Host: localhost\r\n\r\n");
        let mut config = ParserConfig::default();
        
        config.set_header_max_size(1);
        
        parse_headers(&mut src, &config).unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_if_max_num_of_headers_exceed() {
        let mut src = mock_reader(
            b"Host: Localhost\r\n\
            Test-Header: ops..\r\n\
            \r\n"
        );
        let mut config = ParserConfig::default();

        config.set_max_num_headers(1);
        parse_headers(&mut src, &config).unwrap();
    }

    #[test]
    fn multiple_headers() {
        let mut src = mock_reader(
            b"Host: Localhost\r\n\
            Connection: close\r\n\
            Content-Length: 10\r\n\
            It: should work\r\n\r\n"
        );

        parse_headers(&mut src, &ParserConfig::default()).unwrap();
    }
}

