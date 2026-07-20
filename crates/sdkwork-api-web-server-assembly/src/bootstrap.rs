//! Business-only gateway bootstrap for sdkwork-web-server.

use axum::{Extension, Router};
use sdkwork_intelligence_webserver_repository_sqlx::bootstrap_web_runtime_from_env;
use sdkwork_routes_webserver_app_api::{
    gateway_mount as mount_app, wrap_router_with_web_framework_from_env as wrap_app,
};
use sdkwork_routes_webserver_backend_api::{
    gateway_mount as mount_backend, wrap_router_with_web_framework_from_env as wrap_backend,
};
use std::sync::Arc;

pub struct ApiAssembly {
    pub router: Router,
}

pub async fn assemble_api_router() -> Result<ApiAssembly, String> {
    let runtime = bootstrap_web_runtime_from_env().await?;
    let service = Arc::new(runtime.service);
    let app = wrap_app(mount_app(service.clone())).await;
    let backend = wrap_backend(mount_backend(service.clone()), service.clone()).await;
    Ok(ApiAssembly {
        router: Router::new()
            .merge(app)
            .merge(backend)
            .layer(Extension(service)),
    })
}

pub async fn assemble_api_router() -> Result<ApiAssembly, String> {
    assemble_api_router().await
}
