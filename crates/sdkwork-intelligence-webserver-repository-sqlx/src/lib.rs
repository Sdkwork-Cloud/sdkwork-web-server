use sdkwork_database_config::DatabaseEngine;
use sdkwork_database_id::SnowflakeIdGenerator;
use sdkwork_webserver_contract::WebServiceError;
use sqlx::AnyPool;
use std::sync::{Arc, OnceLock};

mod agents;
mod audit;
mod certificates;
mod deployments;
mod domains;
mod domains_lookup;
mod env_variables;
mod health_checks;
mod nginx_configs;
mod port;
mod runtime;
mod servers;
mod sites;
mod support;

pub use runtime::{bootstrap_web_runtime_from_env, WebRuntime};

/// 环境变量机密加密密钥（32 字节 AES-256 key）。
/// 通过 HKDF-SHA256 从 SDKWORK_WEB_SECRET_ENCRYPTION_KEY 派生。
/// 生产环境必须配置；开发环境缺失时使用派生兜底密钥并告警。
pub type SecretEncryptionKey = [u8; 32];

#[derive(Clone)]
pub struct WebRepository {
    pool: AnyPool,
    database_engine: Arc<OnceLock<DatabaseEngine>>,
    id_generator: SnowflakeIdGenerator,
    secret_key: SecretEncryptionKey,
}

impl WebRepository {
    pub fn new(
        pool: AnyPool,
        id_generator: SnowflakeIdGenerator,
        secret_key: SecretEncryptionKey,
    ) -> Self {
        Self {
            pool,
            database_engine: Arc::new(OnceLock::new()),
            id_generator,
            secret_key,
        }
    }

    pub fn new_with_engine(
        pool: AnyPool,
        database_engine: DatabaseEngine,
        id_generator: SnowflakeIdGenerator,
        secret_key: SecretEncryptionKey,
    ) -> Self {
        let engine = OnceLock::new();
        let _ = engine.set(database_engine);
        Self {
            pool,
            database_engine: Arc::new(engine),
            id_generator,
            secret_key,
        }
    }

    pub fn pool(&self) -> &AnyPool {
        &self.pool
    }

    pub fn id_generator(&self) -> &SnowflakeIdGenerator {
        &self.id_generator
    }

    pub fn secret_key(&self) -> &SecretEncryptionKey {
        &self.secret_key
    }

    pub(crate) async fn database_engine(&self) -> Result<DatabaseEngine, WebServiceError> {
        if let Some(engine) = self.database_engine.get() {
            return Ok(*engine);
        }

        let detected = if sqlx::query("SELECT sqlite_version()")
            .fetch_optional(&self.pool)
            .await
            .is_ok()
        {
            DatabaseEngine::Sqlite
        } else if sqlx::query("SELECT current_database()")
            .fetch_optional(&self.pool)
            .await
            .is_ok()
        {
            DatabaseEngine::Postgres
        } else {
            return Err(WebServiceError::Internal(
                "unable to detect repository database engine".to_string(),
            ));
        };
        let _ = self.database_engine.set(detected);
        Ok(*self.database_engine.get().unwrap_or(&detected))
    }
}
