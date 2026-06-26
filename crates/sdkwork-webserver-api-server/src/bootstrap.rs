use axum::{Extension, Router};
use sdkwork_intelligence_webserver_repository_sqlx::bootstrap_web_runtime_from_env;
use sdkwork_routes_webserver_app_api::{
    build_router_with_shared_app_api, wrap_router_with_web_framework_from_env as wrap_app_router,
};
use sdkwork_routes_webserver_backend_api::{
    build_agent_router, build_router_with_shared_backend_api,
    wrap_router_with_web_framework_from_env as wrap_backend_router,
};
use sdkwork_routes_webserver_common::with_problem_correlation;
use sdkwork_web_bootstrap::{service_router, ServiceRouterConfig};
use std::sync::Arc;
use tracing::info;

use crate::readiness::WebServiceReadinessCheck;

pub async fn build_router() -> Result<Router, String> {
    let runtime = bootstrap_web_runtime_from_env().await?;
    info!("Web runtime ready");
    let service = Arc::new(runtime.service);

    let app_business_router = build_router_with_shared_app_api(service.clone());
    let backend_business_router = build_router_with_shared_backend_api(service.clone());
    let agent_router = build_agent_router(service.clone());

    let app_router = wrap_app_router(app_business_router).await;
    let backend_router = wrap_backend_router(backend_business_router).await;

    let business_router = Router::new()
        .merge(app_router)
        .merge(backend_router)
        .merge(with_problem_correlation(agent_router))
        .layer(Extension(service.clone()));

    let service_router_config = ServiceRouterConfig::default()
        .with_readiness_check(Arc::new(WebServiceReadinessCheck::new(service)));

    Ok(service_router(business_router, service_router_config))
}

pub async fn run_database_migrate_only() -> Result<(), String> {
    std::env::set_var("SDKWORK_WEB_DATABASE_AUTO_MIGRATE", "true");
    sdkwork_webserver_database_host::bootstrap_web_database_from_env().await?;
    info!("Web database migration completed");
    Ok(())
}
