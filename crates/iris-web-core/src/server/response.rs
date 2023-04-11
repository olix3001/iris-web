use std::{collections::HashMap, io::Write};

use super::request::Request;

/// Struct that represents a response to a request.
#[derive(Debug, Clone)]
pub struct Response {
    pub status: ResponseStatus,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

/// The status of a response.
#[derive(Debug, Clone)]
pub enum ResponseStatus {
    Ok,
    NotFound,
    BadRequest,
    InternalServerError,
    MethodNotAllowed,
    InvalidRequest,
    Custom(String)
}

impl ResponseStatus {
    /// Returns status code and reason phrase.
    pub fn as_raw(&self) -> String {
        match self {
            ResponseStatus::Ok => "200 OK".to_string(),
            ResponseStatus::NotFound => "404 Not Found".to_string(),
            ResponseStatus::BadRequest => "400 Bad Request".to_string(),
            ResponseStatus::InternalServerError => "500 Internal Server Error".to_string(),
            ResponseStatus::MethodNotAllowed => "405 Method Not Allowed".to_string(),
            ResponseStatus::InvalidRequest => "400 Bad Request".to_string(),
            ResponseStatus::Custom(s) => s.to_string(),

            #[allow(unreachable_patterns)] // For future proofing
            _ => "500 Internal Server Error".to_string(),
        }
    }
}

impl Default for Response {
    fn default() -> Self {
        Self {
            status: ResponseStatus::NotFound,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }
}

impl Response {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_status(mut self, status: ResponseStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_body(mut self, body: impl IntoResponseBody) -> Self {
        self.body = body.into_response_body();
        self
    }

    #[doc(hidden)]
    pub(crate) fn send_response(&mut self, request: &Request) -> std::io::Result<()> {
        let mut response = String::new();

        // Set the content length
        self.headers.insert("Content-Length".to_string(), self.body.len().to_string());

        // Add the status line
        response.push_str(&format!("{} {}\r\n", request.version, self.status.as_raw()));

        // Add the headers
        for (key, value) in &self.headers {
            response.push_str(&format!("{key}: {value}\r\n"));
        }

        // Add the body
        response.push_str("\r\n");
        response.push_str(&String::from_utf8_lossy(&self.body));

        #[cfg(debug_assertions)]
        println!("Response: {response}");

        // Send the response
        let mut stream = request.stream.as_ref().unwrap().lock().unwrap();
        stream.write_all(response.as_bytes())?;
        stream.flush()?;
        stream.shutdown(std::net::Shutdown::Both)?;

        Ok(())
    }
}

pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

/// By default, Iris will serialize the body to JSON, but if you want to send a raw body, you can use this.
pub struct UnserializedBody(pub Vec<u8>);

impl IntoResponseBody for UnserializedBody {
    fn into_response_body(self) -> Vec<u8> {
        self.0
    }
}

impl<T: IntoResponseBody> IntoResponse for T {
    fn into_response(self) -> Response {
        Response {
            status: ResponseStatus::Ok,
            headers: HashMap::new(),
            body: self.into_response_body(),
        }
    }
}

pub trait IntoResponseBody {
    fn into_response_body(self) -> Vec<u8>;
}

impl<T> IntoResponseBody for T where T: serde::Serialize {
    fn into_response_body(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}