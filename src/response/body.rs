//! Provides tools for HTTP message body serialization.

use http_core::Method;

use crate::response::SerializationConfing;

/// The HTTP message body.
/// 
/// Provides methods for message body serialization.
pub(crate) struct MessageBody();

impl MessageBody {
    /// Serializes the HTTP message body and returns 
    /// a tuple containing the length of message body 
    /// and the data, respectively.
    /// 
    /// If the status code is `204`, no body is serialized, but it will 
    /// return a vector contaning the mandatory *`CRLF`* as *data*, and zero
    /// as *length*.
    /// 
    /// If status code is `304` or the method used in the request is 
    /// `HEAD`, body is serialized, but it's not returned. The 
    /// returned value contains the length of message body, and
    /// the mandatory `CRLF`.
    /// 
    /// If none of conditions above is found, the body is serialized
    /// returning the length of message body and the data.
    pub fn serialize(
        body: Option<Vec<u8>>, 
        config: &SerializationConfing
    ) -> (usize, Vec<u8>) {
        let should_serialize = !matches!(config.status.as_str(), "204");
        
        let mut response = Vec::new();
        response.extend_from_slice(b"\r\n");
        
        if should_serialize {
            let is_head = matches!(config.method, Method::HEAD);
            let is_304 = matches!(config.status.as_str(), "304");

            if is_head || is_304 {
                if let Some(body) = body {
                    return (body.len(), response)
                }
            }
            
            if let Some(body) = body {
                response.extend_from_slice(&body);
            }

            return (response.len() - 2, response)
        }

        (0, response)
    }
}

#[cfg(test)]
mod serialization {
    use super::*;
    use http_core::response::StatusCode;

    #[test]
    fn content_match() {
        let content = b"lorem ipsom dolor".to_vec();
        let config  = SerializationConfing {
            method: Method::GET,
            status: StatusCode::Ok.code()
        };

        let res = MessageBody::serialize(Some(content), &config);
        assert_eq!(res.0, 17);
        
        let res = str::from_utf8(res.1.as_slice()).unwrap();
        assert_eq!("\r\nlorem ipsom dolor", res);
        
    }
}