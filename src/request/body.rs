//! Tools for body parsing.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};

use crate::error::ParserErr;
use crate::ParserState;

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
