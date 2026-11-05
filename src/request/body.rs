//! Tools for body parsing.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

use crate::error::{HeaderParserError, ParserErr};
use crate::{ParserConfig, ParserState};
use crate::helpers::find_line;

pub(crate) mod check;

pub(crate) fn read_body<T>(
    src: &mut BufReader<T>,
    headers: Option<&HashMap<String, String>>,
    config: &ParserConfig
) -> Result<ParserState<Option<String>>, ParserErr>
where
    T: Read,
{

    let Some(headers) = headers else {
        return Ok(ParserState::Complete(None));
    };
    let Some(content_length) = headers.get("Content-length") else {
        return Ok(ParserState::Complete(None));
    };

    let content_length = content_length.parse::<usize>().map_err(|err| {
        ParserErr::InvalidInput(err.to_string())
    })?;

    if content_length > config.max_body_size() {
        return Err(ParserErr::TooLong)
    }

    let available_data = src.fill_buf().map_err(|e| ParserErr::Io(e))?;
    let from_buffer = available_data.len().min(content_length);
    let remaining = content_length - from_buffer;
    let mut body = vec![0u8; content_length];

    body[..from_buffer].copy_from_slice(&available_data[..from_buffer]);

    if remaining > 0 {
        src.read_exact(&mut body[from_buffer..])
            .map_err(ParserErr::Io)?;
    }

    src.consume(body.len());

    let body_txt = String::from_utf8(body).map_err(|err| {
        ParserErr::InvalidInput(err.to_string())
    })?;

    Ok(ParserState::Complete(Some(body_txt)))
}

/// Decode and parses a chunked body.
/// 
/// # Errors
/// 
/// Returns an error if the `Transfer-Encoding` header from request
/// is not "chunked". It can change in the future.
/// 
/// If the value of `Transfer-Encoding`  is not UTF-8 or if the value
/// includes  additional characters such as trailing whitespaces an error is returned. 
/// 
///  Returns an *I/O* error if a [`Read`] error is triggered.
pub fn decode_chunked_body<T: Read>(
    src: &mut BufReader<T>,
    headers: Option<&HashMap<String, String>>,
    config: &ParserConfig
)
-> Result<ParserState<Option<String>>, ParserErr>{
    let Some(headers) = headers else {
        return Ok(ParserState::Complete(None))
    };

    let Some(coding) = headers.get("Transfer-Encoding") else {
        return Ok(ParserState::Complete(None))
    };

    if coding.as_str() != "chunked" {
        return Err(ParserErr::InvalidHeader(
            HeaderParserError::InvalidTransferEncoding(coding.to_owned())
        ))
    }

    let mut chunks = vec![];
    let mut length: usize = 0;

    loop {
        let available_content = src.fill_buf().map_err(|e| ParserErr::Io(e))?;
        
        let chunk_size = if let Some(pos) =  find_line(available_content) {
            let content = &available_content[..pos].to_owned();
            src.consume(content.len() + 2);

            let size = std::str::from_utf8(content)
                .map_err(|e| ParserErr::InvalidInput(e.to_string()))?;
            let size = usize::from_str_radix(size, 16).unwrap();

            size 
        } else {
            continue 
        };

        if chunk_size == 0 {
            let content = String::from_utf8(chunks);

            if let Ok(content) = content {
                return Ok(ParserState::Complete(Some(content)));
            } else {
                return Err(ParserErr::InvalidInput(
                    "Invalid body message encoding".into()
                ));
            }
        };
        
        let mut chunk = vec![0u8; chunk_size];
    
        src.read_exact(&mut chunk[..chunk_size]).map_err(|e|  ParserErr::Io(e))?;

        length += chunk.len() + 2;

        if length > config.max_body_size() {
            return Err(ParserErr::TooLong)
        }
        
        chunks.append(&mut chunk);
        src.consume(2);
    }
}

#[cfg(test)]
mod parse_chunked_body {
    use super::*;
    use std::io::Cursor;
    
    fn mock_stream(input: &[u8]) -> BufReader<Cursor<&[u8]>> {
        let mut cursor = Cursor::new(input);
        cursor.set_position(0);
    
        BufReader::new(cursor)
    }
    
    #[test]
    fn decode_successfully() {
        let input = b"5\r\nHello\r\n6\r\n World\r\n0\r\n\r\n";
        let mut headers = HashMap::new();

        headers.insert("Transfer-Encoding".to_string(), "chunked".to_string());
        
        let mut stream = mock_stream(input);

        let res = decode_chunked_body(
            &mut stream, 
            Some(&headers),
            &ParserConfig::default()
        ).unwrap();

        match res {
            ParserState::Complete(k) => {
                let vec = k.unwrap();
                println!("res: {}", vec)
            },
            _ => panic!("Oh my glob")
        }
    }

    #[test]
    #[should_panic]
    fn max_size_exceeded() {
        let mut src = mock_stream(b"5\r\nhello\r\n\r\n");
        let mut config = ParserConfig::default();
        let mut headers = HashMap::new();

        config.set_max_body_size(11);
        
        headers.insert(
            "Transfer-Encoding".to_string(), 
            "chunked".to_string()
        );
        
        decode_chunked_body(&mut src, Some(&headers), &config).unwrap();
    }
}

#[cfg(test)]
mod body_parsing {
    use super::*;
    use std::io::Cursor;
    
    fn mock_stream(input: &[u8]) -> BufReader<Cursor<&[u8]>> {
        let mut cursor = Cursor::new(input);
        cursor.set_position(0);
    
        BufReader::new(cursor)
    }
    
    #[test]
    #[should_panic] 
    fn fail_if_max_size_is_exceeded() {
        let mut src = mock_stream(b"lorem ipsom dolor");
        let mut headers = HashMap::new();
        let mut config = ParserConfig::default();

        config.set_max_body_size(5);

        headers.insert(
            "Content-length".to_string(), 
            "17".to_string()
        );
        
        read_body(&mut src, Some(&headers), &config).unwrap();
    }
}