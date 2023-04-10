use iris_web_core::{router::router::{Router, PathResolver}, server::{http_server::HttpServer, request::Request}, pipeline::pipeline::RequestPipeline};

fn pipeline_test(req: &Request) -> String {
    format!("Request: {:#?}", req)
}

fn main() {
    let mut server = HttpServer::new();
    {
        let mut router = server.get_router_write();
        router.insert("/", PathResolver::Placeholder("Hello world!".to_string()));
        router.add_pipeline("/test", RequestPipeline::controller(pipeline_test));
        println!("{:#?}", router);
    }
    server.listen(("localhost", 8080))
}
