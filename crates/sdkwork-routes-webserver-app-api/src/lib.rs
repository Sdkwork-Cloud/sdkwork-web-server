//! App API route boundary for SDKWork Web Server.

pub mod auth;
pub mod http_route_manifest;
pub mod paths;
pub mod routes;
pub mod web_bootstrap;

pub use http_route_manifest::app_route_manifest;
pub use routes::{build_router_with_app_api, build_router_with_shared_app_api};
pub use sdkwork_webserver_contract::{WebAppApi, WebAppRequestContext};
pub use web_bootstrap::{
    web_app_api_prefixes, web_app_api_public_path_prefixes, wrap_router_with_web_framework_from_env,
};

use sdkwork_web_core::HttpRouteManifest;
use std::sync::Arc;

pub fn gateway_route_manifest() -> HttpRouteManifest {
    app_route_manifest()
}

pub fn gateway_mount(api: Arc<dyn WebAppApi>) -> axum::Router {
    build_router_with_shared_app_api(api)
}
