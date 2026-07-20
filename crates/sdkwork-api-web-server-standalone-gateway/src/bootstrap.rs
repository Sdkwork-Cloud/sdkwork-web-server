use axum::Router;
use sdkwork_api_web_server_assembly::assemble_api_router;
use tracing::info;

pub async fn build_router() -> Result<Router, String> {
    Ok(assemble_api_router().await?.router)
}

pub async fn run_database_migrate_only() -> Result<(), String> {
    std::env::set_var("SDKWORK_WEB_DATABASE_AUTO_MIGRATE", "true");
    sdkwork_webserver_database_host::bootstrap_web_database_from_env().await?;
    info!("Web database migration completed");
    Ok(())
}
