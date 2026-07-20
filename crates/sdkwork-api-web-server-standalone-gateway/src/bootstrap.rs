use axum::{Extension, Router};
use sdkwork_intelligence_webserver_repository_sqlx::bootstrap_web_runtime_from_env;
use sdkwork_routes_webserver_app_api::{
    build_router_with_shared_app_api,
    wrap_router_with_web_framework_from_env_and_metrics as wrap_app_router,
};
use sdkwork_routes_webserver_backend_api::{
    build_router_with_shared_backend_api,
    wrap_router_with_web_framework_from_env_and_metrics as wrap_backend_router,
};
use sdkwork_web_bootstrap::{service_router, ServiceRouterConfig};
use sdkwork_web_core::{HttpMetricsDimensions, HttpMetricsRegistry};
use std::sync::Arc;
use tracing::info;

use crate::{metric_dimensions::CanonicalMetricDimensions, readiness::WebServiceReadinessCheck};

pub async fn build_router() -> Result<Router, String> {
    let metrics = HttpMetricsRegistry::with_dimensions(management_metrics_dimensions_from_env()?);
    let runtime = bootstrap_web_runtime_from_env().await?;
    info!("Web runtime ready");
    let service = Arc::new(runtime.service);

    let app_business_router = build_router_with_shared_app_api(service.clone());
    // Backend router now includes agent routes (C8-C9): heartbeat + sync are
    // registered inside build_router_with_shared_backend_api and authenticated
    // via X-SDKWork-Agent-Token through the WebFrameworkLayer.
    let backend_business_router = build_router_with_shared_backend_api(service.clone());

    let app_router = wrap_app_router(app_business_router, metrics.clone()).await;
    let backend_router =
        wrap_backend_router(backend_business_router, service.clone(), metrics.clone()).await;

    let business_router = Router::new()
        .merge(app_router)
        .merge(backend_router)
        .layer(Extension(service.clone()));

    let service_router_config = ServiceRouterConfig::default()
        .with_readiness_check(Arc::new(WebServiceReadinessCheck::new(service)))
        .with_metrics(metrics);

    Ok(service_router(business_router, service_router_config))
}

fn management_metrics_dimensions_from_env() -> Result<HttpMetricsDimensions, String> {
    let dimensions = CanonicalMetricDimensions::from_env()?;
    let database_engine = std::env::var("SDKWORK_WEB_DATABASE_ENGINE")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .unwrap_or_default();
    let runtime_profile =
        match database_engine.as_str() {
            "" => String::new(),
            "postgres" | "postgresql" => "postgresql".to_owned(),
            "sqlite" => "sqlite".to_owned(),
            _ => return Err(
                "SDKWORK_WEB_DATABASE_ENGINE must be postgresql or sqlite for metrics dimensions"
                    .to_owned(),
            ),
        };
    Ok(HttpMetricsDimensions {
        service: "sdkwork-api-web-server-standalone-gateway".to_owned(),
        environment: dimensions.environment,
        deployment_profile: dimensions.deployment_profile,
        runtime_target: dimensions.runtime_target,
        runtime_profile,
    })
}

pub async fn run_database_migrate_only() -> Result<(), String> {
    std::env::set_var("SDKWORK_WEB_DATABASE_AUTO_MIGRATE", "true");
    sdkwork_webserver_database_host::bootstrap_web_database_from_env().await?;
    info!("Web database migration completed");
    Ok(())
}
