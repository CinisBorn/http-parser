//! The HTTP Response

use std::{collections::HashMap};

use headers::{
    insert_header_with_list_value, 
    insert_header,
    serialize_headers
};

use http_core::Method;
use http_core::response::{
    Response, 
    ResponseData, 
    StatusCode
};

mod headers;
mod body;

/// Contains shared information across serialization processes.
pub(crate) struct SerializationConfing {
    method: Method,
    status: StatusCode
}

impl SerializeResponse for Response {}

/// Define the HTTP response serialization behavior.
pub trait SerializeResponse: ResponseData {
    /// Serializes an HTTP [`Response`].
    fn serialize(&mut self, method: &Method) -> Vec<u8> {
        let config = SerializationConfing {
            method: *method,
            status: self.status().clone()
        };

        let body = self.body().cloned();
        let status = self.status();
        let mut headers = HashMap::new();

        let status_line = sereliaze_status_line("HTTP/1.1", &status.code());

        let body = body::serialize(body.as_ref(), &config);
        let content_length = body.len() - 2;

        if content_length == 0 {
            insert_header(
                &mut headers, 
                "Content-Length", 
                content_length
            );
        }

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

        let response_headers = self.headers();
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
        let status_line = sereliaze_status_line("HTTP/1.1", &self.status().code());

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
    fn to_status(&mut self) -> u16 {
        self.status()
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