use std::{sync::Arc, any::{TypeId, Any}, fmt::Debug};

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
            data: data.clone(),
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