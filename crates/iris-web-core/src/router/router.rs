use std::{collections::HashMap, sync::Mutex};

use crate::{server::{request::Request, response::{Response, ResponseStatus}}, pipeline::request_pipeline::{RequestPipeline, IntoPipeline}, utils::data_container::DataContainer};

use super::Method;

/// A router is a collection of routes that can be used to match a path.
#[derive(Default)]
pub struct Router {
    /// The routes that are registered with this router.
    pub(crate) routes: HashMap<String, PathResolver>,
    /// The fallback route that is used when no other route matches.
    pub(crate) fallback: Option<PathResolver>,

    /// Data that is shared between all routes in this router.
    pub(crate) data: DataContainer,
}

impl Router {
    /// Creates a new router.
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            fallback: None,
            data: DataContainer::default(),
        }
    }

    /// Adds a new route to the router. This is a convenience method for adding a route to the router.
    pub fn add_route<T>(&mut self, path: &str, method: Method, controller: impl IntoPipeline<T>) -> &mut Self {
        self.add_pipeline(path, method, controller.into_pipeline());
        self
    }

    /// Adds new module to the router. This is a convenience method for adding a module to the router.
    pub fn add_module(&mut self, path: &str, module: impl Module) -> &mut Self {
        let mut router = Router::new();

        module.build(&mut router);

        self.insert(path, PathResolver::Router(Box::new(router)));

        self
    }

    /// Adds data to the scoped data container.
    pub fn add_data<T: Send + Sync + 'static>(&mut self, value: T) -> &mut Self {
        self.data.add(value);
        self
    }

    pub(crate) fn add_pipeline(&mut self, path: &str, method: Method, pipeline: RequestPipeline) {
        // Get the resolver or create a new one.
        let resolver = match self.routes.get_mut(path) {
            Some(resolver) => resolver,
            None => {
                let mut pipeline_map = HashMap::new();
                pipeline_map.insert(method.as_str(), Mutex::new(pipeline));
                self.insert(path, PathResolver::Pipeline(pipeline_map));
                return;
            }
        };

        // Add the pipeline to the resolver.
        #[allow(clippy::single_match)]
        match resolver {
            PathResolver::Pipeline(pipelines) => {
                pipelines.insert(method.as_str(), Mutex::new(pipeline));
            }
            _ => {}
        }

        #[cfg(debug_assertions)]
        println!("{self:#?}");
    }

    /// Inserts a new route into the router creating sub-routers as needed.
    /// :id can be used like a placeholder to match any path segment.
    pub(crate) fn insert(&mut self, path: &str, resolver: PathResolver) {
        // Special case for root path.
        if path.trim() == "/" {
            self.routes.insert("".to_string(), resolver);
            return;
        }

        let mut segments = path.split('/').filter(|s| !s.is_empty());

        // Get the first segment of the path.
        let segment = match segments.next() {
            Some(segment) => segment,
            None => return,
        };

        // Get the rest of the path.
        let rest = segments.collect::<Vec<_>>().join("/");

        // If rest is empty, then insert the resolver directly.
        if rest.is_empty() {
            // Placeholder
            if segment.starts_with(':') {
                // Insert as fallback.
                self.fallback = Some(resolver);
            } else {
                self.routes.insert(segment.to_string(), resolver);
            }
            return;
        }

        // If the segment is a placeholder, then insert new router as fallback.
        if segment.starts_with(':') {
            // Check if there is already a fallback.
            match self.fallback {
                Some(PathResolver::Router(_)) => {}
                Some(_) => {
                    // Create a new router and insert the existing resolver as an "" route.
                    let mut router = Router::new();
                    router.routes.insert("".to_string(), self.fallback.take().unwrap());
                    router.insert(&rest, resolver);
                    self.fallback = Some(PathResolver::Router(Box::new(router)));
                }
                None => {
                    // Create a new router.
                    let mut router = Router::new();
                    router.insert(&rest, resolver);
                    self.fallback = Some(PathResolver::Router(Box::new(router)));
                }
            }
        } else {
            let sub_router = match self.routes.get_mut(segment) {
                Some(PathResolver::Router(router)) => router,
                Some(_) => {
                    // Create a new router and insert the existing resolver as an "" route.
                    let mut router = Router::new();
                    router.routes.insert("".to_string(), self.routes.remove(segment).unwrap());
                    self.routes.insert(segment.to_string(), PathResolver::Router(Box::new(router)));
                    match self.routes.get_mut(segment) {
                        Some(PathResolver::Router(router)) => router,
                        _ => unreachable!(),
                    }
                }
                None => {
                    // Create a new router.
                    let router = Router::new();
                    self.routes.insert(segment.to_string(), PathResolver::Router(Box::new(router)));
                    match self.routes.get_mut(segment) {
                        Some(PathResolver::Router(router)) => router,
                        _ => unreachable!(),
                    }
                }
            };

            sub_router.insert(&rest, resolver);
        }
    }

    /// Finds a route that matches the given path and returns the resolver.
    /// :id can be used like a placeholder to match any path segment.
    pub fn resolve(&self, path: &str) -> Option<(&PathResolver, DataContainer)> {
        self.resolve_internal(path, DataContainer::default())
    } 
    
    #[doc(hidden)]
    fn resolve_internal(&self, path: &str, current_data: DataContainer) -> Option<(&PathResolver, DataContainer)> {
        let mut segments = path.split('/').filter(|s| !s.is_empty());

        let data = current_data.combine(&self.data);

        // Get the first segment of the path.
        let segment = segments.next().unwrap_or("");

        // Get the rest of the path.
        let rest = segments.collect::<Vec<_>>().join("/");

        if let Some(resolver) = self.routes.get(segment) {
            match resolver {
                PathResolver::Router(ref router) => router.resolve_internal(&rest, data),
                _ => Some((resolver, data)),
            }
        } else {
            match self.fallback {
                Some(PathResolver::Router(ref router)) => router.resolve_internal(&rest, data),
                Some(_) => {
                    if rest.is_empty() {
                        self.fallback.as_ref().map(|r| (r, data))
                    } else {
                        None
                    }
                }
                None => None,
            }
        }
    }
}

// Debug impl
impl std::fmt::Debug for Router {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Router")
            .field("routes", &self.routes)
            .field("fallback", &self.fallback)
            .finish()
    }
}

#[derive(Debug)]
/// Enum that represents all possible ways to resolve a path.
pub enum PathResolver {
    Router(Box<Router>),
    Placeholder(String),
    Pipeline(HashMap<String, Mutex<RequestPipeline>>),
}

impl PartialEq for PathResolver {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PathResolver::Placeholder(a), PathResolver::Placeholder(b)) => a == b,
            _ => false,
        }
    }
}

impl PathResolver {
    /// Returns new response based on the request
    pub fn resolve(&self, request: &Request, data: DataContainer) -> Response {
        match self {
            PathResolver::Placeholder(data) => Response::new().with_status(ResponseStatus::Ok).with_body(data.clone().into_bytes()),
            PathResolver::Pipeline(pipeline) => {
                // Get the method
                let method = &request.method;

                // Get the pipeline
                let pipeline = match pipeline.get(method) {
                    Some(pipeline) => pipeline,
                    None => return Response::new().with_status(ResponseStatus::MethodNotAllowed)
                };

                // Get the pipeline
                let mut pipeline = pipeline.lock().unwrap();

                // Resolve the pipeline
                pipeline.handle(request.clone(), data)
            }
            _ => Response::new().with_status(ResponseStatus::InternalServerError),
        }
    }
}

pub trait Module {
    fn build(self, router: &mut Router);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_router() {
        let mut router = Router::new();

        router.insert("/", PathResolver::Placeholder("Root".to_string()));
        router.insert("/hello/world", PathResolver::Placeholder("Hello World".to_string()));
        router.insert("/hello/world/test", PathResolver::Placeholder("Hello World test".to_string()));

        router.insert("/hello/:name", PathResolver::Placeholder("Hello Name".to_string()));
        router.insert("/hello/:name/:age", PathResolver::Placeholder("Hello Name Age".to_string()));

        assert_eq!(router.resolve("/").unwrap().0, &PathResolver::Placeholder("Root".to_string()));
        assert_eq!(router.resolve("/hello/world").unwrap().0, &PathResolver::Placeholder("Hello World".to_string()));
        assert_eq!(router.resolve("/hello/world/test").unwrap().0, &PathResolver::Placeholder("Hello World test".to_string()));
        assert_eq!(router.resolve("/hello/John").unwrap().0, &PathResolver::Placeholder("Hello Name".to_string()));
        assert_eq!(router.resolve("/hello/John/20").unwrap().0, &PathResolver::Placeholder("Hello Name Age".to_string()));
    }
}