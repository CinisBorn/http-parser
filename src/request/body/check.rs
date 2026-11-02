use crate::error::{HeaderParserError, ParserErr};
use std::collections::HashMap;

pub(crate) enum BodyProcessMethod {
    TransferCoding,
    ContentLength,
    None,
}

pub(crate) fn body_framing_method(
    headers: Option<&HashMap<String, String>>,
) -> Result<BodyProcessMethod, ParserErr> {
    if let Some(headers) = headers {
        let is_transfer = headers.contains_key("Transfer-Encoding");
        let is_content = headers.contains_key("Content-Length");

        if is_content && is_transfer {
            return Err(ParserErr::InvalidHeader(
                HeaderParserError::ConflictingBodyFraming,
            ));
        }

        if is_content {
            return Ok(BodyProcessMethod::ContentLength);
        } else {
            return Ok(BodyProcessMethod::TransferCoding);
        }
    } else {
        return Ok(BodyProcessMethod::None);
    }
}

#[cfg(test)]
mod framing {
    use super::*;

    #[test]
    fn return_correct_framing_method() {
        let mut headers = HashMap::new();

        headers.insert("Content-Length".to_string(), "12".to_string());

        match body_framing_method(Some(headers).as_ref()).unwrap() {
            BodyProcessMethod::ContentLength => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    #[should_panic]
    fn conflict_is_error() {
        let mut headers = HashMap::new();

        headers.insert("Content-Length".to_string(), "12".to_string());
        headers.insert("Transfer-Encoding".to_string(), "chunked".to_string());

        body_framing_method(Some(headers).as_ref()).unwrap();
    }

    #[test]
    fn header_not_exists() {
        let res = body_framing_method(None).unwrap();

        match res {
            BodyProcessMethod::None => assert!(true),
            _ => assert!(false),
        }
    }
}
