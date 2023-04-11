pub mod router;
pub mod server;
pub mod utils;
pub mod pipeline;

pub mod prelude {
    // Server
    pub use crate::server::http_server::HttpServer;

    // Request
    pub use crate::server::request::Request;
    pub use crate::server::response::Response;
    pub use crate::server::response::UnserializedBody;

    // Router
    pub use crate::router::router::{Router, Module};
    pub use crate::router::PathParams;

    // Data-related
    pub use crate::pipeline::controller::Data;

    // Methods
    pub use crate::router::Method;

    // Pipeline
    pub use crate::pipeline::request_pipeline::PipelineData;
    pub use crate::pipeline::controller::ConfigurableController;
}