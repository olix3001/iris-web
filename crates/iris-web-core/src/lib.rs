pub mod router;
pub mod server;
pub mod utils;
pub mod pipeline;

pub mod prelude {
    // Server
    pub use crate::server::http_server::HttpServer;

    // Router
    pub use crate::router::router::{Router, Module};

    // Data-related
    pub use crate::pipeline::controller::Data;

    // Methods
    pub use crate::router::Method;
}