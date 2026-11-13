//! Provides tools for HTTP message body serialization.

use http_core::Method;

use crate::response::SerializationConfing;

pub(crate) fn serialize(
    body: Option<Vec<u8>>, 
    config: &SerializationConfing
) -> Vec<u8> {
    let is_allowed = body_is_allowed(
        config.status.as_str(), 
        config.method
    );

    let mut response = Vec::new();
    response.extend_from_slice(b"\r\n");

    if let Some(body) = body && is_allowed {
        response.extend_from_slice(&body);
    }

    response
}

/// Check if it's allowed the response insert a message body.
pub(crate) fn body_is_allowed(status: &str, method: Method) -> bool {
    let method = !matches!(method, Method::HEAD);
    let status = !matches!(status, "304" | "204");

    method && status
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

        let res = serialize(Some(content), &config);
        let res = str::from_utf8(res.as_slice()).unwrap();

        assert_eq!("\r\nlorem ipsom dolor", res);
    }
}