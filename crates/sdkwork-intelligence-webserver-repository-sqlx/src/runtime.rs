//! Web runtime bootstrap: database lifecycle + repository + service assembly.

use sdkwork_database_config::DatabaseConfig;
use sdkwork_database_sqlx::create_any_pool_from_config;
use sdkwork_database_id::SnowflakeIdGenerator;
use sdkwork_intelligence_webserver_service::WebService;
use sdkwork_utils_rust::derive_aes_256_key;
use sdkwork_webserver_acme_service::CertificateIssuer;
use sdkwork_webserver_database_host::bootstrap_web_database_from_env;
use sdkwork_webserver_edge_runtime::EdgeRuntime;
use sqlx::AnyPool;
use std::sync::Arc;

use sdkwork_intelligence_webserver_service::WebRepositoryPort;

use crate::{SecretEncryptionKey, WebRepository};

/// HKDF info 上下文绑定，将派生密钥绑定到环境变量机密加密用途。
const ENV_SECRET_KEY_INFO: &[u8] = b"sdkwork-web-env-variable-encryption";

/// Bootstrapped Web application runtime.
pub struct WebRuntime {
    pub service: WebService,
}

fn is_production() -> bool {
    matches!(
        std::env::var("SDKWORK_WEB_ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .to_ascii_lowercase()
            .as_str(),
        "production" | "prod"
    )
}

/// Snowflake node_id 从环境变量加载，多实例部署时必须显式配置以避免 ID 碰撞。
fn snowflake_from_env() -> Result<SnowflakeIdGenerator, String> {
    let node_id = match std::env::var("SDKWORK_WEB_SNOWFLAKE_NODE_ID") {
        Ok(value) => value
            .parse::<u16>()
            .map_err(|error| format!("invalid SDKWORK_WEB_SNOWFLAKE_NODE_ID: {error}"))?,
        Err(_) => {
            return Err(
                "SDKWORK_WEB_SNOWFLAKE_NODE_ID is required (multi-instance must set unique node id)"
                    .to_string(),
            )
        }
    };
    SnowflakeIdGenerator::new(node_id).map_err(|error| error.to_string())
}

/// 加载环境变量机密加密密钥。
/// 生产环境必须配置 SDKWORK_WEB_SECRET_ENCRYPTION_KEY；开发环境缺失时使用派生兜底密钥。
fn secret_key_from_env() -> Result<SecretEncryptionKey, String> {
    let use_production = is_production();
    let raw = match std::env::var("SDKWORK_WEB_SECRET_ENCRYPTION_KEY") {
        Ok(value) => value,
        Err(_) if !use_production => {
            tracing::warn!(
                "SDKWORK_WEB_SECRET_ENCRYPTION_KEY missing; using development-only derived key"
            );
            "sdkwork-web-development-secret-key".to_string()
        }
        Err(_) => {
            return Err(
                "SDKWORK_WEB_SECRET_ENCRYPTION_KEY is required in production environments".to_string(),
            )
        }
    };
    Ok(derive_aes_256_key(
        raw.as_bytes(),
        b"sdkwork-web-env",
        ENV_SECRET_KEY_INFO,
    ))
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
    let secret_key = secret_key_from_env()?;
    let repository =
        Arc::new(WebRepository::new(pool, id_generator, secret_key)) as Arc<dyn WebRepositoryPort>;

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
