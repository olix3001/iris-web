use iris_web_core::router::router::{Router, PathResolver};

fn main() {
    let mut router = Router::new();

    router.insert("/hello/world", PathResolver::Placeholder("Hello World".to_string()));
    router.insert("/hello/world/test", PathResolver::Placeholder("Hello World test".to_string()));

    router.insert("/hello/:name", PathResolver::Placeholder("Hello Name".to_string()));
    router.insert("/hello/:name/:age", PathResolver::Placeholder("Hello Name Age".to_string()));

    println!("{:#?}", router);

    println!("{:?}", router.resolve("/hello/world").unwrap());
    println!("{:?}", router.resolve("/hello/world/test").unwrap());
    println!("{:?}", router.resolve("/hello/John").unwrap());
    println!("{:?}", router.resolve("/hello/John/20").unwrap());
}
