//! Provides tools for headers handling.
//! 
//! This module is design to parse, test, check, format and etc for response 
//! serialization.
//! 
//! check "headers_sections" at request folder for parsing logic.

use std::{collections::HashMap, fmt::Display};

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

#[cfg(test)]
mod serialization {
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
}