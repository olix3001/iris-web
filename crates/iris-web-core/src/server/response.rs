use std::{collections::HashMap, io::Write};

use super::request::Request;

/// Struct that represents a response to a request.
pub struct Response {
    pub status: ResponseStatus,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

/// The status of a response.
pub enum ResponseStatus {
    Ok,
    NotFound,
    BadRequest,
    InternalServerError,
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
            ResponseStatus::Custom(s) => s.to_string(),
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

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    #[doc(hidden)]
    pub(crate) fn send_response(&self, request: &Request) -> std::io::Result<()> {
        let mut response = String::new();

        // Add the status line
        response.push_str(&format!("{} {}\r\n", request.version, self.status.as_raw()));

        // Add the headers
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }

        // Add the body
        response.push_str("\r\n");
        response.push_str(&String::from_utf8_lossy(&self.body));

        println!("Response: {}", response);

        // Send the response
        let mut stream = request.stream.as_ref().unwrap().lock().unwrap();
        stream.write_all(response.as_bytes())?;
        stream.flush()?;
        stream.shutdown(std::net::Shutdown::Both)?;

        Ok(())
    }
}
