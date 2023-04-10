use iris_web_core::{router::router::{Router, PathResolver}, server::http_server::HttpServer};

fn main() {
    let mut server = HttpServer::new();
    {
        let mut router = server.get_router_write();
        router.insert("/", PathResolver::Placeholder("Hello world!".to_string()));
        println!("{:#?}", router);
    }
    server.listen(("localhost", 8080))
}
