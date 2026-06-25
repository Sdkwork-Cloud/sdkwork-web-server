//! Web runtime bootstrap: database lifecycle + repository + service assembly.

use sdkwork_database_config::DatabaseConfig;
use sdkwork_database_sqlx::create_any_pool_from_config;
use sdkwork_id_core::SnowflakeIdGenerator;
use sdkwork_intelligence_webserver_service::WebService;
use sdkwork_webserver_acme_service::CertificateIssuer;
use sdkwork_webserver_database_host::bootstrap_web_database_from_env;
use sdkwork_webserver_edge_runtime::EdgeRuntime;
use sqlx::AnyPool;
use std::sync::Arc;

use sdkwork_intelligence_webserver_service::WebRepositoryPort;

use crate::WebRepository;

/// Bootstrapped Web application runtime.
pub struct WebRuntime {
    pub service: WebService,
}

fn snowflake_from_env() -> Result<SnowflakeIdGenerator, String> {
    let node_id = std::env::var("SDKWORK_WEB_SNOWFLAKE_NODE_ID")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(1);
    SnowflakeIdGenerator::new(node_id).map_err(|error| error.to_string())
}

async fn any_pool_from_env() -> Result<AnyPool, String> {
    let _ = dotenvy::dotenv();
    let config = DatabaseConfig::from_env("WEB")
        .map_err(|error| format!("read Web database config failed: {error}"))?;
    create_any_pool_from_config(config)
        .await
        .map_err(|error| format!("create Web any pool failed: {error}"))
}

/// Bootstrap database lifecycle, repository, and service from environment variables.
pub async fn bootstrap_web_runtime_from_env() -> Result<WebRuntime, String> {
    bootstrap_web_database_from_env().await?;
    let pool = any_pool_from_env().await?;
    let id_generator = snowflake_from_env()?;
    let repository = Arc::new(WebRepository::new(pool, id_generator)) as Arc<dyn WebRepositoryPort>;

    let certificate_issuer = Arc::new(
        CertificateIssuer::from_env()
            .map_err(|error| format!("certificate issuer bootstrap failed: {error}"))?,
    );
    let edge_runtime = Arc::new(
        EdgeRuntime::from_env()
            .map_err(|error| format!("edge runtime bootstrap failed: {error}"))?,
    );

    Ok(WebRuntime {
        service: WebService::new(repository, certificate_issuer, edge_runtime),
    })
}
