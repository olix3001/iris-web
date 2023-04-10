use std::{net::TcpListener, sync::{Arc, RwLock}};

use crate::{router::router::Router, utils::thread_pool::ThreadPool, server::{request::Request, response::Response}};

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

    /// Gets the router for the server (this should not be used by the user).
    pub fn get_router_write(&self) -> std::sync::RwLockWriteGuard<Router> {
        self.router.write().unwrap()
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
                            Some(path_resolver) => {
                                println!("Resolver: {:?}", path_resolver);
                                path_resolver.resolve(&request).send_response(&request).unwrap()
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
