use std::{net::TcpListener, sync::{Arc, RwLock}};

use crate::{router::{router::{Router, Module}, Method}, utils::{thread_pool::ThreadPool, data_container::DataContainer}, server::{request::Request, response::Response}, pipeline::{pipeline::{IntoPipeline, RequestPipeline}, controller::{Controller, IntoController}}};

pub type BindAddress<'a> = (&'a str, u16);

/// Basic HTTP server implementation with support for TLS.
pub struct HttpServer {
    pub router: Arc<RwLock<Router>>,
    pub(crate) thread_pool: ThreadPool,

    #[doc(hidden)]
    listener: Option<TcpListener>,
}

impl HttpServer {
    /// Creates a new HTTP server.
    /// The server will not start listening until `listen` is called.
    /// The server will use the default thread pool size of 4.
    pub fn new() -> Self {
        Self {
            router: Arc::new(RwLock::new(Router::new())),
            thread_pool: ThreadPool::new(4),
            listener: None,
        }
    }

    pub fn dump_routes(&mut self) -> &mut Self {
        println!("{:#?}", self.router.read().unwrap());
        self
    }

    /// Gets the router for the server (this should not be used by the user).
    pub fn get_router_write(&self) -> std::sync::RwLockWriteGuard<Router> {
        self.router.write().unwrap()
    }

    /// Adds new routes to the server.
    /// This is a convenience method for adding routes to the router.
    pub fn add_route<T>(&mut self, path: &str, method: Method, controller: impl IntoPipeline<T>) -> &mut Self {
        {
            let mut router_write = self.router.write().unwrap();
            router_write.add_route(path, method, controller);
        }
        self
    }

    /// Adds new module to the server.
    pub fn add_module(&mut self, path: &str, module: impl Module) -> &mut Self {
        {
            let mut router_write = self.router.write().unwrap();
            router_write.add_module(path, module);
        }
        self
    }

    /// Adds data to the global data container.
    pub fn add_data<T: Send + Sync + 'static>(&mut self, value: T) -> &mut Self {
        self.router.write().unwrap().data.add(value);
        self
    }

    /// Starts listening for incoming connections on the specified address.
    pub fn listen(&mut self, address: BindAddress) {
        let listener = TcpListener::bind(address).unwrap();
        self.listener = Some(listener);

        #[cfg(debug_assertions)]
        println!("Listening on {}:{}", address.0, address.1);

        for stream in self.listener.as_ref().unwrap().incoming() {
            match stream {
                Ok(stream) => {
                    let router = self.router.clone();
                    self.thread_pool.queue(move || {
                        println!("New connection: {}", stream.peer_addr().unwrap());

                        // Parse the request.
                        let request = Request::from_stream(stream);

                        // Get the path resolver from the router.
                        let router_read = router.read().unwrap();
                        let path_resolver = router_read.resolve(&request.path);

                        match path_resolver {
                            Some((path_resolver, path_data)) => {
                                println!("Resolver: {:?}", path_resolver);
                                println!("Path data: {:?}", path_data);

                                path_resolver.resolve(&request, path_data).send_response(&request).unwrap();
                            }
                            None => {
                                println!("No path resolver found for path: {}", request.path);
                                Response::default().send_response(&request).unwrap()
                            }
                        }
                    });
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    }
}