//! Internal API route boundary for Website runtime-set distribution.

pub mod auth;
pub mod http_route_manifest;
pub mod paths;
pub mod routes;
pub mod web_bootstrap;

pub use http_route_manifest::internal_route_manifest;
pub use routes::{build_router_with_internal_api, build_router_with_shared_internal_api};
pub use sdkwork_webserver_contract::{WebInternalApi, WebInternalRequestContext};
pub use web_bootstrap::{
    wrap_router_with_web_framework_from_env, wrap_router_with_web_framework_from_env_and_metrics,
};

use sdkwork_web_core::HttpRouteManifest;
use std::sync::Arc;

pub fn gateway_route_manifest() -> HttpRouteManifest {
    internal_route_manifest()
}

pub fn gateway_mount(api: Arc<dyn WebInternalApi>) -> axum::Router {
    build_router_with_shared_internal_api(api)
}
