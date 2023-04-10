use iris_web_core::{router::{router::{Router, PathResolver}, Method}, server::{http_server::HttpServer, request::Request}, pipeline::{pipeline::RequestPipeline, controller::Data}};

fn pipeline_test(data: Data<HelloWorld>) -> String {
    data.test.clone()
}

fn test(req: &Request) -> String {
    format!("Request path: {:?}", req.path)
}

#[derive(Debug)]
struct HelloWorld {
    test: String,
}

fn main() {
    HttpServer::new()
        .add_data(HelloWorld { test: "Hello World!".to_string() })
        .add_route("/", Method::GET, pipeline_test)
        .add_route("/:test", Method::GET, test)
        .listen(("localhost", 8080));
}
