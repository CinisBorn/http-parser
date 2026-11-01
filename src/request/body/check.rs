use std::collections::HashMap;
use crate::error::{ParserErr, HeaderParserError};

pub(crate) enum BodyProcessMethod {
    TransferCoding,
    ContentLength,
    None 
}

pub(crate) fn body_framing_method(headers: &Option<HashMap<String, String>>) -> Result<BodyProcessMethod, ParserErr> {
    if let Some(headers) = headers {
        let is_transfer = headers.contains_key("Transfer-Coding");
        let is_content = headers.contains_key("Content-Length");

        if is_content && is_transfer {
            return Err(ParserErr::InvalidHeader(HeaderParserError::ConflictingBodyFraming))
        }

        if is_content { 
            return Ok(BodyProcessMethod::ContentLength)
        } else { 
            return Ok(BodyProcessMethod::TransferCoding) 
        }
    } else {
        return Ok(BodyProcessMethod::None)
    }
}

