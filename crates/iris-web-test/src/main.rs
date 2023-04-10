use std::sync::atomic::AtomicU32;

use iris_web_core::{prelude::*, pipeline::{request_pipeline::PipelineData, controller::ConfigurableController}};

fn test() -> String {
    "Hello World!".to_string()
}

fn router_test() -> String {
    "Hello Router!".to_string()
}

fn router_test_count(counter: Data<Counter>) -> String {
    let value = counter.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    format!("Counter: {}", value)
}

fn middleware_test(data: &mut PipelineData) {
    data.add_data("Hello Middleware!".to_string());
}

struct TestRouter;

impl Module for TestRouter {
    fn build(self, router: &mut Router) {
        router
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
        .add_module("/:test", TestRouter)
        .dump_routes()
        .listen(("localhost", 8080));
}
