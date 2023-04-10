use std::marker::PhantomData;
use crate::server::response::IntoResponse;
use super::controller::{Controller, ControllerParam, IntoController};
use super::request_pipeline::PipelineData;
use crate::server::response::Response;

/// Wrapper for a function that can be used as a controller.
pub struct FunctionController<Input, F> {
    f: F,
    marker: PhantomData<fn() -> Input>
}

impl<Input, F> FunctionController<Input, F> {
    /// Creates a new `FunctionController` instance.
    pub fn new(f: F) -> Self {
        Self {
            f,
            marker: Default::default()
        }
    }
}

macro_rules! impl_controller {
    ($($params:ident),*) => {
        #[allow(non_snake_case, unused)]
        #[doc(hidden)]
        impl<
            R: IntoResponse, F, $($params: ControllerParam),*
        > Controller for FunctionController<($($params,)*), F>
            where
                for<'a, 'b> &'a mut F:
                    FnMut( $($params),* ) -> R +
                    FnMut( $(<$params as ControllerParam>::Item<'b>),* ) -> R,
        {
            fn handle(&mut self, pipeline: &mut PipelineData) -> Response {
                // Without this rustc complains without reason
                fn call_inner<R: IntoResponse, $($params),*>(
                    mut f: impl FnMut($($params),*) -> R,
                    $($params: $params),*
                ) -> R {
                    f($($params),*)
                }

                // Get the data from the request pipeline
                $(
                    let $params = $params::fetch(pipeline).unwrap();
                )*

                // Call the function
                call_inner(&mut self.f, $($params),*).into_response()
            }
        }
    }
}

macro_rules! impl_into_controller {
    ($($params:ident),*) => {
        #[allow(non_snake_case, unused)]
        #[doc(hidden)]
        impl<
            R: IntoResponse, F, $($params: ControllerParam),*
        > IntoController<($($params,)*)> for F
        where
            for<'a, 'b> &'a mut F:
                FnMut( $($params),* ) -> R +
                FnMut( $(<$params as ControllerParam>::Item<'b>),* ) -> R
        {
            type Controller = FunctionController<($($params,)*), F>;

            fn into_controller(self) -> Self::Controller {
                FunctionController {
                    f: self,
                    marker: Default::default()
                }
            }
        }
    }
}

impl_controller!();
impl_controller!(A0);
impl_controller!(A0, A1);
impl_controller!(A0, A1, A2);
impl_controller!(A0, A1, A2, A3);

impl_into_controller!();
impl_into_controller!(A0);
impl_into_controller!(A0, A1);
impl_into_controller!(A0, A1, A2);
impl_into_controller!(A0, A1, A2, A3);
