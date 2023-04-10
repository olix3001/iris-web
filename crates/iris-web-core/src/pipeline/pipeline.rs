use std::{collections::HashMap, any::{TypeId, Any}, sync::Arc, fmt::Debug};

use crate::server::{request::Request, response::Response};

use super::controller::{Controller, IntoController};

pub(crate) type BoxedController = Box<dyn Controller + Send + Sync>;

pub struct RequestPipeline {
    // pub(crate) middlewares: Vec<Box<dyn Middleware>>,
    pub(crate) controller: BoxedController,
}

impl Debug for RequestPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestPipeline")
            // .field("middlewares", &self.middlewares)
            // .field("controller", &self.controller)
            .finish()
    }
}

impl RequestPipeline {
    pub fn new(controller: BoxedController) -> Self {
        Self {
            // middlewares: vec![],
            controller,
        }
    }

    pub fn controller<I, C: Controller + Send + Sync + 'static>(controller: impl IntoController<I, Controller = C>) -> Self {
        Self::new(Box::new(controller.into_controller()))
    }

    pub fn handle(&mut self, request: Request) -> Response {
        let pipeline = PipelineData::new(request);

        self.controller.handle(&pipeline)
    }
}

pub struct PipelineData {
    pub request: Request,
    
    pub data: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl PipelineData {
    pub fn new(request: Request) -> Self {
        Self {
            request,
            data: HashMap::new(),
        }
    }
}