//! Web runtime bootstrap: database lifecycle + repository + service assembly.

use std::sync::Arc;

use sdkwork_database_id::SnowflakeIdGenerator;
use sdkwork_database_sqlx::{enable_process_shared_database_pool, DatabasePool};
use sdkwork_intelligence_webserver_service::{WebRepositoryPort, WebService};
use sdkwork_utils_rust::derive_aes_256_key;
use sdkwork_webserver_acme_service::{
    AcmeConfig, CertificateIssuer, DEFAULT_ACME_OPERATION_TIMEOUT_MS,
};
use sdkwork_webserver_contract::{web_environment_name, web_is_production_like_environment};
use sdkwork_webserver_database_host::bootstrap_web_database_from_env;
use sdkwork_webserver_edge_runtime::EdgeRuntime;

use crate::{PostgresWebRepository, SecretEncryptionKey, SqliteWebRepository};

const ENV_SECRET_KEY_INFO: &[u8] = b"sdkwork-web-env-variable-encryption";

/// Bootstrapped Web application runtime.
pub struct WebRuntime {
    pub service: WebService,
}

fn snowflake_from_env() -> Result<SnowflakeIdGenerator, String> {
    let node_id = match std::env::var("SDKWORK_WEB_SNOWFLAKE_NODE_ID") {
        Ok(value) => value
            .parse::<u16>()
            .map_err(|error| format!("invalid SDKWORK_WEB_SNOWFLAKE_NODE_ID: {error}"))?,
        Err(_) => {
            return Err(
                "SDKWORK_WEB_SNOWFLAKE_NODE_ID is required (multi-instance must set unique node id)"
                    .to_string(),
            );
        }
    };
    SnowflakeIdGenerator::new(node_id).map_err(|error| error.to_string())
}

fn secret_key_from_env() -> Result<SecretEncryptionKey, String> {
    let production_like = web_is_production_like_environment();
    let raw = match std::env::var("SDKWORK_WEB_SECRET_ENCRYPTION_KEY") {
        Ok(value) => value,
        Err(_) if !production_like => {
            tracing::warn!(
                "SDKWORK_WEB_SECRET_ENCRYPTION_KEY missing; using development-only derived key"
            );
            "sdkwork-web-development-secret-key".to_string()
        }
        Err(_) => {
            return Err(
                "SDKWORK_WEB_SECRET_ENCRYPTION_KEY is required in production-like environments"
                    .to_string(),
            );
        }
    };
    Ok(derive_aes_256_key(
        raw.as_bytes(),
        b"sdkwork-web-env",
        ENV_SECRET_KEY_INFO,
    ))
}

fn certificate_issuer_from_env() -> Result<CertificateIssuer, String> {
    let environment = web_environment_name();
    let environment_production_like = web_is_production_like_environment();
    let use_production = match std::env::var("SDKWORK_WEB_ACME_PROFILE") {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "production" | "prod" => true,
            "staging" | "stage" | "test" => false,
            other => {
                return Err(format!(
                    "invalid SDKWORK_WEB_ACME_PROFILE {other}; expected production or staging"
                ));
            }
        },
        Err(_) => matches!(environment.as_str(), "production" | "prod"),
    };
    let production_like = environment_production_like || use_production;
    let directory_url = std::env::var("SDKWORK_WEB_ACME_DIRECTORY_URL").unwrap_or_else(|_| {
        if use_production {
            "https://acme-v02.api.letsencrypt.org/directory".to_string()
        } else {
            "https://acme-staging-v02.api.letsencrypt.org/directory".to_string()
        }
    });
    let contact_email = match std::env::var("SDKWORK_WEB_ACME_CONTACT_EMAIL") {
        Ok(value) => value,
        Err(_) if !production_like => "admin@localhost".to_string(),
        Err(_) => {
            return Err(
                "SDKWORK_WEB_ACME_CONTACT_EMAIL is required in production-like environments"
                    .to_string(),
            );
        }
    };
    let renew_before_days = parse_env_or("SDKWORK_WEB_CERT_RENEW_BEFORE_DAYS", 30_u32)?;
    let webroot = std::env::var("SDKWORK_WEB_ACME_WEBROOT").ok();
    let encryption_key = match std::env::var("SDKWORK_WEB_CERT_ENCRYPTION_KEY") {
        Ok(value) => value,
        Err(_) if !production_like => {
            tracing::warn!(
                "SDKWORK_WEB_CERT_ENCRYPTION_KEY missing; using development-only derived key"
            );
            "sdkwork-web-development-cert-key".to_string()
        }
        Err(_) => {
            return Err(
                "SDKWORK_WEB_CERT_ENCRYPTION_KEY is required in production-like environments"
                    .to_string(),
            );
        }
    };
    let cert_root = std::env::var("SDKWORK_WEB_CERT_LIVE_ROOT")
        .unwrap_or_else(|_| "/opt/certs/letsencrypt/live".to_string());
    let operation_timeout_ms = parse_env_or(
        "SDKWORK_WEB_ACME_OPERATION_TIMEOUT_MS",
        DEFAULT_ACME_OPERATION_TIMEOUT_MS,
    )?;

    let config = AcmeConfig::new(
        directory_url,
        contact_email,
        renew_before_days,
        webroot,
        encryption_key.as_bytes(),
        use_production,
        production_like,
    )
    .map_err(|error| format!("ACME configuration failed: {error}"))?;
    CertificateIssuer::new_with_operation_timeout_ms(config, cert_root, operation_timeout_ms)
        .map_err(|error| format!("certificate issuer bootstrap failed: {error}"))
}

fn parse_env_or<T>(key: &str, default: T) -> Result<T, String>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    std::env::var(key)
        .map(|value| {
            value
                .parse::<T>()
                .map_err(|error| format!("invalid {key}: {error}"))
        })
        .unwrap_or(Ok(default))
}

/// Bootstrap database lifecycle, repository, and service from environment variables.
pub async fn bootstrap_web_runtime_from_env() -> Result<WebRuntime, String> {
    enable_process_shared_database_pool();
    let lifecycle_host = bootstrap_web_database_from_env().await?;
    let id_generator = snowflake_from_env()?;
    let secret_key = secret_key_from_env()?;
    let repository = match lifecycle_host.pool() {
        DatabasePool::Postgres(pool, _) => Arc::new(PostgresWebRepository::new(
            pool.clone(),
            sdkwork_database_config::DatabaseEngine::Postgres,
            id_generator,
            secret_key,
        )) as Arc<dyn WebRepositoryPort>,
        DatabasePool::Sqlite(pool, _) => Arc::new(SqliteWebRepository::new(
            pool.clone(),
            sdkwork_database_config::DatabaseEngine::Sqlite,
            id_generator,
            secret_key,
        )) as Arc<dyn WebRepositoryPort>,
    };

    let certificate_issuer = Arc::new(certificate_issuer_from_env()?);
    let edge_runtime = Arc::new(
        EdgeRuntime::from_env()
            .map_err(|error| format!("edge runtime bootstrap failed: {error}"))?,
    );

    Ok(WebRuntime {
        service: WebService::new(repository, certificate_issuer, edge_runtime),
    })
}
