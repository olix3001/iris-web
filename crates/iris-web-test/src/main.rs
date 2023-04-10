use iris_web_core::{prelude::*, pipeline::{request_pipeline::PipelineData, controller::ConfigurableController}};

fn test() -> String {
    "Hello World!".to_string()
}

fn router_test(string: Data<String>) -> String {
    println!("Data: {:?}", string);
    "Hello Router!".to_string()
}

fn middleware_test(data: &mut PipelineData) {
    data.add_data("Hello Middleware!".to_string());
}

struct TestRouter;

impl Module for TestRouter {
    fn build(self, router: &mut Router) {
        router
            .add_route("/", Method::GET, router_test.with_middleware(middleware_test))
            .add_route("/test", Method::POST, router_test)
            .add_route("/test", Method::GET, || "Hello Test!".to_string());
    }
}

fn main() {
    HttpServer::new()
        .add_route("/", Method::GET, test)
        .add_module("/:test", TestRouter)
        .dump_routes()
        .listen(("localhost", 8080));
}
