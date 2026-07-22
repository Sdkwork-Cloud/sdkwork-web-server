//! Business-only gateway bootstrap for sdkwork-web-server.

use axum::{Extension, Router};
use sdkwork_intelligence_webserver_repository_sqlx::bootstrap_web_runtime_from_env;
use sdkwork_intelligence_webserver_service::WebService;
use sdkwork_routes_webserver_app_api::{
    gateway_mount as mount_app, wrap_router_with_web_framework_from_env as wrap_app,
};
use sdkwork_routes_webserver_backend_api::{
    gateway_mount as mount_backend, wrap_router_with_web_framework_from_env as wrap_backend,
};
use sdkwork_routes_webserver_internal_api::{
    gateway_mount as mount_internal, wrap_router_with_web_framework_from_env as wrap_internal,
};
use sdkwork_webserver_contract::MachineCredentialAuthenticator;
use std::sync::Arc;

pub struct ApiAssembly {
    pub router: Router,
    pub service: Arc<WebService>,
}

pub async fn assemble_business_routes() -> Result<ApiAssembly, String> {
    let runtime = bootstrap_web_runtime_from_env().await?;
    let service = Arc::new(runtime.service);
    let app = wrap_app(mount_app(service.clone())).await;
    let backend = wrap_backend(mount_backend(service.clone()), service.clone()).await;
    let machine_authenticator: Arc<dyn MachineCredentialAuthenticator> = service.clone();
    let internal = wrap_internal(mount_internal(service.clone()), machine_authenticator).await;
    Ok(ApiAssembly {
        router: Router::new()
            .merge(app)
            .merge(backend)
            .merge(internal)
            .layer(Extension(service.clone())),
        service,
    })
}

pub async fn assemble_api_router() -> Result<ApiAssembly, String> {
    assemble_business_routes().await
}
