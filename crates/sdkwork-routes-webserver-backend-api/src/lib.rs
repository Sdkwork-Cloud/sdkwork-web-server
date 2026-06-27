//! Backend API route boundary for SDKWork Web Server.

pub mod agent_routes;
pub mod auth;
pub mod http_route_manifest;
pub mod paths;
pub mod routes;
pub mod web_bootstrap;

pub use http_route_manifest::backend_route_manifest;
pub use routes::{build_router_with_backend_api, build_router_with_shared_backend_api};
pub use sdkwork_webserver_contract::{WebBackendApi, WebBackendRequestContext};
pub use web_bootstrap::wrap_router_with_web_framework_from_env;
