use sdkwork_database_id::SnowflakeIdGenerator;
use sqlx::AnyPool;

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
}
