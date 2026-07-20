use axum::Router;
use sdkwork_api_web_server_assembly::assemble_api_router;
use sdkwork_web_bootstrap::{service_router, ServiceRouterConfig};
use std::sync::Arc;
use tracing::info;

use crate::readiness::WebServiceReadinessCheck;

pub async fn build_router() -> Result<Router, String> {
    let assembly = assemble_api_router().await?;
    let readiness = Arc::new(WebServiceReadinessCheck::new(assembly.service));
    Ok(service_router(
        assembly.router,
        ServiceRouterConfig::default().with_readiness_check(readiness),
    ))
}

pub async fn run_database_migrate_only() -> Result<(), String> {
    std::env::set_var("SDKWORK_WEB_DATABASE_AUTO_MIGRATE", "true");
    sdkwork_webserver_database_host::bootstrap_web_database_from_env().await?;
    info!("Web database migration completed");
    Ok(())
}
