use iris_web_core::{router::{router::{Router, PathResolver}, Method}, server::{http_server::HttpServer, request::Request}, pipeline::pipeline::RequestPipeline};

fn pipeline_test(req: &Request) -> String {
    format!("Request headers: {:?}", req.headers)
}
fn test(req: &Request) -> String {
    format!("Request path: {:?}", req.path)
}

fn main() {
    HttpServer::new()
        .add_route("/", Method::GET, pipeline_test)
        .add_route("/:test", Method::GET, test)
        .listen(("localhost", 8080));
}
