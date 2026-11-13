//! Provides tools for status line serialization. 

/// Serializes the status line of Response without the code phrasal.
/// ```ignore
/// # use http_core::response::StatusCode;
/// # use http_parser::response::status::serialize;
/// let res = serialize("HTTP/1.1", &StatusCode::Ok.code());
/// let res = str::from_utf8(res.as_slice()).unwrap();
/// 
/// assert_eq!("HTTP/1.1 200\r\n", res);
/// ```
pub(crate) fn serialize(version: &str, status: &str) -> Vec<u8> {
    let mut response = Vec::new();
    response.extend_from_slice(version.as_bytes());
    response.extend_from_slice(b" ");
    response.extend_from_slice(status.as_bytes());
    response.extend_from_slice(b"\r\n");

    response
}

#[cfg(test)]
mod serialization {
    use super::*;
    use http_core::response::StatusCode;

    #[test]
    fn content_match() {
        let res = serialize("HTTP/1.1", &StatusCode::Ok.code());
        let res = str::from_utf8(res.as_slice()).unwrap();

        assert_eq!("HTTP/1.1 200\r\n", res);
    }
}