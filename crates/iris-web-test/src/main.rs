use std::sync::atomic::AtomicU32;

use iris_web_core::{prelude::*, pipeline::{request_pipeline::PipelineData, controller::ConfigurableController}, server::request::Request};
use iris_web_json::json_body::json_body;
use serde::{Deserialize, Serialize};

fn test() -> String {
    "Hello World!".to_string()
}

fn router_test(data: Data<String>) -> String {
    format!("Data from middleware: {}", data.data)
}

fn router_test_count(counter: Data<Counter>) -> String {
    let value = counter.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    format!("Counter: {}", value)
}

fn middleware_test(data: &mut PipelineData) -> Option<()> {
    data.add_data("Hello Middleware!".to_string());
    None
}

fn router_test_body(request: &Request, body: Data<TestBody>) -> TestBody {
    TestBody {
        test: body.data.test.clone(),
        value: body.data.value + 1,
        path: Some(request.path.clone()),
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct TestBody {
    test: String,
    value: u32,
    path: Option<String>,
}

struct TestModule;

impl Module for TestModule {
    fn build(self, router: &mut Router) {
        router
            .add_route("/body", Method::POST, router_test_body.with_middleware(json_body::<TestBody>))
            .add_route("/", Method::GET, router_test.with_middleware(middleware_test))
            .add_route("/count", Method::GET, router_test_count)
            .add_route("/test", Method::GET, || "Hello Test!".to_string());
    }
}

struct Counter {
    count: AtomicU32,
}

fn main() {
    HttpServer::new()
        .add_data(Counter {
            count: AtomicU32::new(0),
        })
        .add_route("/", Method::GET, test)
        .add_module("/:test", TestModule)
        .dump_routes()
        .listen(("localhost", 8080));
}
