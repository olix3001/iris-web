use std::{collections::HashMap, sync::Mutex, fmt::Debug};

#[allow(clippy::module_inception)]
pub mod router;

pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

impl Method {
    pub fn as_str(&self) -> String {
        match self {
            Method::GET => "GET".to_string(),
            Method::POST => "POST".to_string(),
            Method::PUT => "PUT".to_string(),
            Method::DELETE => "DELETE".to_string(),
            Method::PATCH => "PATCH".to_string(),
            Method::HEAD => "HEAD".to_string(),
            Method::OPTIONS => "OPTIONS".to_string(),
            Method::CONNECT => "CONNECT".to_string(),
            Method::TRACE => "TRACE".to_string(),
        }
    }
}

pub struct PathParams {
    #[doc(hidden)]
    pub params: Mutex<HashMap<String, String>>,
}

impl Debug for PathParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PathParams")
            .field("params", &self.params.lock().unwrap())
            .finish()
    }
}

impl PathParams {
    pub(crate) fn new() -> Self {
        Self {
            params: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn add_param(&self, key: String, value: String) {
        self.params.lock().unwrap().insert(key, value);
    }

    pub fn get_param(&self, key: &str) -> Option<String> {
        self.params.lock().unwrap().get(key).cloned()
    }

    pub fn get_params(&self) -> HashMap<String, String> {
        self.params.lock().unwrap().clone()
    }
}