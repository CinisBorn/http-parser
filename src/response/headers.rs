//! Provides tools for headers handling.
//! 
//! This module is design to parse, test, check, format and etc for response 
//! serialization.
//! 
//! check "headers_sections" at request folder for parsing logic.

use std::{collections::HashMap, fmt::Display};

use http_core::response::StatusCode;

use crate::response::{SerializationConfing, body::body_is_allowed};

pub(crate) fn insert_header_with_list_value<T: AsRef<str>> (
    headers: &mut HashMap<String, String>,
    key: &str,
    values: Vec<T>
) {
    let mut value = String::new();

    for v in values {
        value.push_str(v.as_ref());
        value.push_str(",");
    }

    value.pop();

    headers.insert(key.to_string(), value);
}

pub(crate) fn insert_header<T: Display>(
    headers: &mut HashMap<String, String>,
    key: &str, 
    value: T,
) {
    headers.insert(
        key.to_string(), 
        value.to_string()
    );
}

pub(crate) fn serialize_headers(
    headers: &mut HashMap<String, String>,
    response_headers: &HashMap<String, String>,
) -> Vec<u8> {
    let mut response = Vec::new();

    for (key, value) in headers {
        response.extend_from_slice(key.as_bytes());
        response.extend_from_slice(b": ");
        response.extend_from_slice(value.as_bytes());
        response.extend_from_slice(b"\r\n");
    }

    if !response_headers.is_empty() {
        for (key, value) in response_headers {
            response.extend_from_slice(key.as_bytes());
            response.extend_from_slice(b": ");
            response.extend_from_slice(value.as_bytes());
            response.extend_from_slice(b"\r\n");
        }
    }

    response
}

/// Inserts and initializates the `Content-Length` header. If the response 
/// doesn't have a message body, a `Content-Length` with value of `0` is 
/// inserted. 
/// 
/// Note that some methods and status code doesn't allow the presence
/// of `Content-Length`. In theses cases, `Content-Length` isn't inserted.
pub(crate) fn content_length(
    length: usize, 
    headers: &mut HashMap<String, String>,
    config: &SerializationConfing
) {
    let res = body_is_allowed(&config.status, config.method);
    
    if res {
        insert_header(headers, "Content-Length", length);
    }
}

/// Inserts the `Allow` header with a list of supported 
/// methods if the server know the method, but the 
/// target resource doesn't support it.
pub(crate) fn method_not_allowed(status: &StatusCode, headers: &mut HashMap<String, String>) {
    if let StatusCode::MethodNotAllowed(methods) = status {
        let methods: Vec<String> = methods
            .iter()
            .cloned()
            .map(|v| v.to_str().to_string())
            .collect();
        
        insert_header_with_list_value(headers, "Allow", methods);
    }
}

#[cfg(test)]
mod serialization {
    use http_core::Method;
    use super::*;

    #[test]
    fn insert_correctly() {
        let mut hashmap: HashMap<String, String> = HashMap::new();

        insert_header_with_list_value(
            &mut hashmap, 
            "test", 
            vec!["it", "works", "fine"]
        );

        let mut value = hashmap.get("test").unwrap().split(",");

        assert_eq!(value.next().unwrap(), "it");
        assert_eq!(value.next().unwrap(), "works");
        assert_eq!(value.next().unwrap(), "fine");
    }

    #[test]
    fn insert_header_successfully() {
        let mut headers: HashMap<String, String> = HashMap::new();
        insert_header(&mut headers, "Content-Length", 10);

        assert_eq!(headers.get("Content-Length").unwrap(), "10");
    }

    #[test]
    fn headers_match() {
        let mut headers: HashMap<String, String> = HashMap::new();
        let methods = Vec::from([Method::GET]);
        let config = &SerializationConfing { 
            method: Method::GET, 
            status: StatusCode::MethodNotAllowed(methods.clone()).code() 
        };
        
        content_length(10, &mut headers, config);
        method_not_allowed(
            &StatusCode::MethodNotAllowed(methods), 
            &mut headers
        );

        assert!(headers.contains_key("Content-Length"));
        assert!(headers.contains_key("Allow"));
    }

    #[test]
    fn header_are_not_inserted() {
        let mut headers: HashMap<String, String> = HashMap::new();
        let config = &SerializationConfing { 
            method: Method::HEAD, 
            status: StatusCode::NoContent.code() 
        };
        
        content_length(0, &mut headers, config);
        method_not_allowed(
            &StatusCode::UriTooLong, 
            &mut headers
        );

        assert!(!headers.contains_key("Content-Length"));
        assert!(!headers.contains_key("Allow"));
    }
}