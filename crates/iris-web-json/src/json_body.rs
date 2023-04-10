use std::fmt::Debug;

use iris_web_core::{pipeline::request_pipeline::PipelineData, server::response::{Response, ResponseStatus, IntoResponse, IntoResponseBody}};
use serde::Serialize;

/// Middleware to parse the request body as JSON.
pub fn raw_json_body(pipeline: &mut PipelineData) -> Option<Response> {
    let content_type = pipeline.request.headers.get("Content-Type");
    if content_type.is_none() {
        return Some(Response::new()
                    .with_status(ResponseStatus::InvalidRequest)
                    .with_body("Missing Content-Type header"));
    }
    let content_type = content_type.unwrap();
    if !content_type.starts_with("application/json") {
        return Some(Response::new()
                    .with_status(ResponseStatus::InvalidRequest)
                    .with_body("Invalid Content-Type header"));
    }
    let body = String::from_utf8_lossy(&pipeline.request.body);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    pipeline.add_data(json);

    None
}

/// Middleware to parse the request body from JSON into a struct.
pub fn json_body<T: serde::de::DeserializeOwned + Send + Sync + Debug + 'static>(pipeline: &mut PipelineData) -> Option<Response> {
    let content_type = pipeline.request.headers.get("Content-Type");
    if content_type.is_none() {
        return Some(Response::new()
                    .with_status(ResponseStatus::InvalidRequest)
                    .with_body("Missing Content-Type header"));
    }
    let content_type = content_type.unwrap();
    if !content_type.starts_with("application/json") {
        return Some(Response::new()
                    .with_status(ResponseStatus::InvalidRequest)
                    .with_body("Invalid Content-Type header"));
    }
    let body = String::from_utf8_lossy(&pipeline.request.body);
    let json: Result<T, serde_json::Error> = serde_json::from_str(&body);
    if json.is_err() {
        return Some(Response::new()
                    .with_status(ResponseStatus::InvalidRequest)
                    .with_body(format!("Invalid JSON body: {}", json.unwrap_err())));
    }

    pipeline.add_data::<T>(json.unwrap());

    None
}