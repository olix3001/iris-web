use std::{fmt::Debug, sync::Arc};

use crate::{server::{request::Request, response::Response}, utils::data_container::DataContainer};

use super::{controller::{Controller, IntoController}, commands::CommandQueue, middleware::BoxedMiddlewareHandler};

pub(crate) type BoxedController = Box<dyn Controller + Send + Sync>;

pub struct RequestPipeline {
    pub(crate) middlewares: Vec<BoxedMiddlewareHandler>,
    pub(crate) controller: BoxedController,
}

impl Debug for RequestPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestPipeline")
            .field("middleware count", &self.middlewares.len())
            .finish()
    }
}

impl RequestPipeline {
    pub fn new(controller: BoxedController) -> Self {
        Self {
            middlewares: Vec::new(),
            controller,
        }
    }

    pub fn controller<I, C: Controller + Send + Sync + 'static>(controller: impl IntoController<I, Controller = C>) -> Self {
        Self::new(Box::new(controller.into_controller()))
    }

    pub fn handle(&mut self, request: Request, data: DataContainer) -> Response {
        let mut pipeline = PipelineData::new(request, data);

        for middleware in &mut self.middlewares {
            middleware.handle(&mut pipeline);

            // Execute all commands in the queue
            pipeline.command_queue.clone().execute(&mut pipeline);
        }

        self.controller.handle(&mut pipeline)   
    }
}

pub struct PipelineData {
    pub request: Request,
    pub(crate) command_queue: Arc<CommandQueue>,
    
    pub(crate) data: DataContainer,
}

impl PipelineData {
    pub fn new(request: Request, initial_data: DataContainer) -> Self {
        Self {
            request,
            command_queue: Arc::new(CommandQueue::new()),
            data: initial_data,
        }
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.data.get::<T>()
    }

    pub fn add_data<T: Send + Sync + 'static>(&mut self, data: T) {
        self.data.add(data);
    }
}

pub trait IntoPipeline<M> {
    fn into_pipeline(self) -> RequestPipeline;
}

impl<T, I, C: Controller + Send + Sync + 'static> IntoPipeline<(I, C)> for T
    where T: IntoController<I, Controller = C>
{
    fn into_pipeline(self) -> RequestPipeline {
        RequestPipeline::controller(self)
    }
}