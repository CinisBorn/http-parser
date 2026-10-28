//! The HTTP Response

use std::{collections::HashMap};

use headers::insert_header_with_list_value;

use http_core::Method;
use http_core::response::{
    Response, 
    ResponseData, 
    StatusCode
};

mod headers;

impl SerializeResponse for Response {}

/// Define the HTTP response serialization behavior.
pub trait SerializeResponse: ResponseData {
    /// Serializes an HTTP [`Response`].
    fn serialize(&mut self, method: &Method) -> Vec<u8> {
        let body = self.get_body().cloned();
        let status = self.get_status().to_owned();
        let mut headers = HashMap::new();

        let status_line = sereliaze_status_line("HTTP/1.1", &status.code());
        let serialize_body = check_response(&status.code(), method);

        let body = insert_body(body.as_ref(), serialize_body);
        let content_length = body.len() - 2;

        insert_content_length(&mut headers, serialize_body, content_length);

        if let StatusCode::MethodNotAllowed(methods) = status {
            insert_header_with_list_value(
                &mut headers, 
                "Allow", 
                methods
                    .iter()
                    .cloned()
                    .map(|v| v.to_str().to_string())
                    .collect::<Vec<String>>()
            );
        }

        let response_headers = self.get_headers();
        let headers = serialize_headers(&mut headers, response_headers);
        let mut response = Vec::new();

        response.extend_from_slice(&status_line);
        response.extend_from_slice(&headers);
        response.extend_from_slice(&body);

        response
    }

    /// Serializes an HTTP response with a status code `503` (Service Unavailable).
    fn service_unavailable(&self) -> Vec<u8> {
        let mut response = Vec::new();
        let status_line = sereliaze_status_line("HTTP/1.1", &self.get_status().code());

        response.extend_from_slice(&status_line);
        response.extend_from_slice(b"\r\n");

        response
    }

    #[cfg(debug_assertions)]
    #[allow(clippy::missing_panics_doc)]
    /// Serializes a HTTP response and convert into `String`.
    fn serialize_to_string(&mut self, method: &Method) -> String {
        String::from_utf8(self.serialize(method)).unwrap()
    }

    /// Parses [`String`] status code into [`u16`].
    fn status(&mut self) -> u16 {
        self.get_status()
            .code()
            .parse::<u16>()
            .expect("parse status code")
    }
}

fn sereliaze_status_line(version: &str, status: &str) -> Vec<u8> {
    let mut response = Vec::new();
    response.extend_from_slice(version.as_bytes());
    response.extend_from_slice(b" ");
    response.extend_from_slice(status.as_bytes());
    response.extend_from_slice(b"\r\n");

    response
}

fn serialize_headers(
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

fn insert_body(body: Option<&Vec<u8>>, serialize_body: bool) -> Vec<u8> {
    let mut response = Vec::new();
    // mandatory body break line
    response.extend_from_slice(b"\r\n");

    if let Some(body) = body && serialize_body {
        response.extend_from_slice(body.as_slice());
    }

    response
}

/// Checks if the response will have a body.
fn check_response(status: &str, method: &Method) -> bool {
    let method = !matches!(method, Method::HEAD);
    let status = !matches!(status, "304" | "204");

    method && status
}

/// Inserts content length header conditionally
fn insert_content_length(
    headers: &mut HashMap<String, String>,
    serialize_body: bool,
    content_length: usize,
) {
    if serialize_body {
        headers.insert("Content-Length".to_string(), content_length.to_string());
    }
}