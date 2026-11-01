//! Tools for body parsing.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};

use crate::error::{HeaderParserError, ParserErr};
use crate::ParserState;
use crate::helpers::find_line;

pub(crate) mod check;

pub(crate) fn read_body<T>(
    src: &mut BufReader<T>,
    headers: Option<&HashMap<String, String>>,
) -> Result<ParserState<Option<String>>, ParserErr>
where
    T: Read + Write,
{
    let Some(headers) = headers else {
        return Ok(ParserState::Complete(None));
    };
    let Some(content_length) = headers.get("Content-length") else {
        return Ok(ParserState::Complete(None));
    };

    let Ok(content_length) = content_length.parse::<usize>() else {
        return Err(ParserErr::InvalidInput(content_length.to_owned()));
    };

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

    let Ok(body_txt) = String::from_utf8(body) else {
        return Err(ParserErr::InvalidInput(String::from("No Valid Input")));
    };

    Ok(ParserState::Complete(Some(body_txt)))
}

/// Decode and parses a chunked body.
/// 
/// # Errors
/// 
/// Returns an error if the `Transfer-Coding` header from request
/// is not "chunked". It can change in the future.
/// 
/// If the value of `Transfer-Coding`  is not UTF-8 or if the value
/// includes  additional characters such as trailing whitespaces an error is returned. 
/// 
///  Returns an *I/O* error if a [`Read`] error is triggered.
pub fn decode_chunked_body<T: Read>(
    src: &mut BufReader<T>,
    headers: Option<&HashMap<String, String>>,)
-> Result<ParserState<Option<String>>, ParserErr>{
    let Some(headers) = headers else {
        return Ok(ParserState::Complete(None))
    };

    let Some(coding) = headers.get("Transfer-Coding") else {
        return Ok(ParserState::Complete(None))
    };

    if coding.as_str() != "chunked" {
        return Err(ParserErr::InvalidHeader(
            HeaderParserError::InvalidTransferEncoding(coding.to_owned())
        ))
    }

    let mut chunks = vec![];

    loop {
        let available_content = src.fill_buf().map_err(|e| ParserErr::Io(e))?;
        
        let size = if let Some(pos) =  find_line(available_content) {
            let content = &available_content[..pos].to_owned();
            src.consume(content.len() + 2);

            let size = std::str::from_utf8(content).map_err(|e| ParserErr::InvalidInput(e.to_string()))?;
            let size = usize::from_str_radix(size, 16).unwrap();

            size 
        } else {
            continue 
        };

        if size == 0 {
            return Ok(ParserState::Complete(Some(String::from_utf8(chunks).unwrap())));
        }
        
        let mut chunk = vec![0u8; size];
    
        src.read_exact(&mut chunk[..size]).map_err(|e|  ParserErr::Io(e))?;

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

        headers.insert("Transfer-Coding".to_string(), "chunked".to_string());
        
        let mut stream = mock_stream(input);

        let res = decode_chunked_body(&mut stream, Some(&headers)).unwrap();

        match res {
            ParserState::Complete(k) => {
                let vec = k.unwrap();
                println!("res: {}", vec)
            },
            _ => panic!("Oh my glob")
        }
    }
}