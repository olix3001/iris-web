use std::{sync::Arc, any::TypeId, fmt::Debug};

use crate::server::{response::{Response}, request::Request};

use super::{request_pipeline::{PipelineData, BoxedController, IntoPipeline, RequestPipeline}, middleware::{BoxedMiddlewareHandler, IntoMiddleware, MiddlewareHandler}};

/// A controller is a function that takes a request and returns a response.
pub trait Controller {
    fn handle(&mut self, pipeline: &PipelineData) -> Response;
}

/// Parameter which can be used in a controller.
pub trait ControllerParam {
    type Item<'new>;

    fn fetch<'r>(pipeline: &'r PipelineData) -> Option<Self::Item<'r>>;
}

pub trait IntoController<Input> {
    type Controller: Controller;

    fn into_controller(self) -> Self::Controller;
}

impl ControllerParam for &PipelineData {
    type Item<'new> = &'new PipelineData;

    fn fetch<'r>(pipeline: &'r PipelineData) -> Option<Self::Item<'r>> {
        Some(pipeline)
    }
}

/// Most basic way that can be used to get a parameter from a request.
pub struct Data<'a, T: 'static> {
    #[allow(dead_code)]
    pub data: Arc<T>,
    marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Clone for Data<'a, T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T: 'static> Data<'a, T> {
    pub fn get_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
    pub fn get_type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

impl<'a, T: Debug> Debug for Data<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl<'a, T: Send + Sync + 'static> ControllerParam for Data<'a, T> {
    type Item<'new> = Data<'new, T>;

    fn fetch<'r>(pipeline: &'r PipelineData) -> Option<Self::Item<'r>> {
        let data = pipeline.data.get::<T>();

        data.map(|data| Data {
            data,
            marker: std::marker::PhantomData,
        })
    }
}

// Allow data to be used without using .data
impl<'a, T: 'static> std::ops::Deref for Data<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data.as_ref()
    }
}

impl ControllerParam for &Request {
    type Item<'new> = &'new Request;

    fn fetch<'r>(pipeline: &'r PipelineData) -> Option<Self::Item<'r>> {
        Some(&pipeline.request)
    }
}

pub trait ConfigurableController<T> {
    fn with_middleware<I, M: MiddlewareHandler + Send + Sync + 'static>(self, middleware: impl IntoMiddleware<I, Middleware = M>) -> ConfiguredController;
}

impl<T, I, C: Controller + Sync + Send + 'static> ConfigurableController<(I, C)> for T where T: IntoController<I, Controller = C> {
    fn with_middleware<I0, M: MiddlewareHandler + Send + Sync + 'static>(self, middleware: impl IntoMiddleware<I0, Middleware = M>) -> ConfiguredController {
        ConfiguredController {
            controller: Box::new(self.into_controller()),
            middlewares: vec![Box::new(middleware.into_middleware())],
        }
    }
}

pub struct ConfiguredController {
    pub controller: BoxedController,
    pub middlewares: Vec<BoxedMiddlewareHandler>,
}

impl ConfiguredController {
    pub fn with_middleware<I, M: MiddlewareHandler + Send + Sync + 'static>(mut self, middleware: impl IntoMiddleware<I, Middleware = M>) -> ConfiguredController {
        self.middlewares.push(Box::new(middleware.into_middleware()));
        self
    }
}

impl IntoPipeline<ConfiguredController> for ConfiguredController {
    fn into_pipeline(self) -> RequestPipeline {
        RequestPipeline {
            controller: self.controller,
            middlewares: self.middlewares,
        }
    }
}