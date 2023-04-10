use super::request_pipeline::PipelineData;
use crate::pipeline::controller::ControllerParam;
use crate::server::response::{IntoResponse, Response};

pub trait MiddlewareHandler {
    fn handle(&mut self, data: &mut PipelineData) -> Option<Response>;
}

pub(crate) type BoxedMiddlewareHandler = Box<dyn MiddlewareHandler + Send + Sync>;

pub struct FunctionMiddleware<Input, F> {
    f: F,
    pmarker: std::marker::PhantomData<fn() -> Input>,
}

pub trait IntoMiddleware<Input> {
    type Middleware: MiddlewareHandler;

    fn into_middleware(self) -> Self::Middleware;
}

impl<Input, F> FunctionMiddleware<Input, F> {
    pub fn new(f: F) -> Self {
        Self {
            f,
            pmarker: std::marker::PhantomData,
        }
    }
}

#[allow(non_snake_case, unused)]
#[doc(hidden)]
impl<R: IntoResponse, F> MiddlewareHandler for FunctionMiddleware<&mut PipelineData, F>
where
    for<'a, 'b> &'a mut F: FnMut(&'b mut PipelineData) -> Option<R>,
{
    fn handle(&mut self, pipeline: &mut PipelineData) -> Option<Response> {
        // Without this rustc complains without reason
        fn call_inner<R: IntoResponse>(mut f: impl FnMut(&mut PipelineData) -> Option<R>, pipeline: &mut PipelineData) -> Option<R> 
        {
            f(pipeline)
        }

        // Call the function
        call_inner(&mut self.f, pipeline).map(|r| r.into_response())
    }
}

#[allow(non_snake_case, unused)]
#[doc(hidden)]
impl<'c, F, R: IntoResponse> IntoMiddleware<&'c mut PipelineData> for F
where
    for<'a, 'b> &'a mut F: FnMut(&'b mut PipelineData) -> Option<R>,
{
    type Middleware = FunctionMiddleware<&'c mut PipelineData, F>;

    fn into_middleware(self) -> Self::Middleware {
        FunctionMiddleware::new(self)
    }
}

macro_rules! impl_middleware {
    ($($param:ident),*) => {
        #[allow(non_snake_case, unused)]
        #[doc(hidden)]
        impl<
            R: IntoResponse,
            F, $($param: ControllerParam),*
        > MiddlewareHandler for FunctionMiddleware<($($param,)*), F>
            where
                for<'a, 'b> &'a mut F:
                    FnMut( $($param),* ) -> Option<R> +
                    FnMut( $(<$param as ControllerParam>::Item<'b>),* ) -> Option<R>,
        {
            fn handle(&mut self, pipeline: &mut PipelineData) -> Option<Response> {
                // Without this rustc complains without reason
                fn call_inner<R: IntoResponse, $($param),*>(
                    mut f: impl FnMut($($param),*) -> Option<R>,
                    $($param: $param),*
                ) -> Option<R> {
                    f($($param),*)
                }

                // Get the data from the request pipeline
                $(
                    let $param = $param::fetch(pipeline).unwrap();
                )*

                // Call the function
                call_inner(&mut self.f, $($param),*).map(|r| r.into_response())
            }
        }
    };
}

macro_rules! impl_into_middleware {
    ($($param:ident),*) => {
        #[allow(non_snake_case, unused)]
        #[doc(hidden)]
        impl<
            R: IntoResponse,
            F, $($param: ControllerParam),*
        > IntoMiddleware<($($param,)*)> for F
            where
                for<'a, 'b> &'a mut F:
                    FnMut( $($param),* ) -> Option<R> +
                    FnMut( $(<$param as ControllerParam>::Item<'b>),* ) -> Option<R>,
        {
            type Middleware = FunctionMiddleware<($($param,)*), F>;

            fn into_middleware(self) -> Self::Middleware {
                FunctionMiddleware::new(self)
            }
        }
    };
}

impl_middleware!();
impl_middleware!(A0);
impl_middleware!(A0, A1);
impl_middleware!(A0, A1, A2);
impl_middleware!(A0, A1, A2, A3);

impl_into_middleware!();
impl_into_middleware!(A0);
impl_into_middleware!(A0, A1);
impl_into_middleware!(A0, A1, A2);
impl_into_middleware!(A0, A1, A2, A3);