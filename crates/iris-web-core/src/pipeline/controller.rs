use std::{sync::Arc, any::{TypeId, Any}};

use crate::server::{response::{IntoResponse, Response}, request::Request};

use super::pipeline::PipelineData;

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

impl<'a, T: 'static> ControllerParam for Data<'a, T> {
    type Item<'new> = Data<'new, T>;

    fn fetch<'r>(pipeline: &'r PipelineData) -> Option<Self::Item<'r>> {
        let data = pipeline.data.get(&TypeId::of::<Arc<T>>()).and_then(|data| data.downcast_ref::<Arc<dyn Any + Send + Sync>>());

        match data {
            Some(data) => Some(Data { data: data.downcast_ref::<Arc<T>>().unwrap().clone(), marker: Default::default() }),
            None => None
        }
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