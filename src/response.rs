//! The HTTP Response

use std::{collections::HashMap};

use headers::serialize_headers;

use http_core::Method;
use http_core::response::{
    Response, 
    ResponseData
};

use crate::response::headers::{content_length, method_not_allowed};

mod headers;
mod body;
mod status;

/// Contains shared information across serialization processes.
pub(crate) struct SerializationConfing {
    method: Method,
    status: String
}

impl SerializeResponse for Response {}

/// Define the HTTP response serialization behavior.
pub trait SerializeResponse: ResponseData {
    /// Serializes an HTTP [`Response`].
    fn serialize(&mut self, method: &Method) -> Vec<u8> {
        let config = SerializationConfing {
            method: *method,
            status: self.status().code()
        };

        let mut headers = HashMap::new();

        let status_line = status::serialize("HTTP/1.1", &config.status);
        let body = body::serialize(self.body().cloned(), &config);

        // initializes some headers if needed.
        content_length(body.len() - 2, &mut headers, &config);
        method_not_allowed(self.status(), &mut headers);
        
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
        let status_line = status::serialize("HTTP/1.1", &self.status().code());

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