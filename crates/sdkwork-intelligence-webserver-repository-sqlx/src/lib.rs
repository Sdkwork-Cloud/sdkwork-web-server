use sdkwork_id_core::SnowflakeIdGenerator;
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

#[derive(Clone)]
pub struct WebRepository {
    pool: AnyPool,
    id_generator: SnowflakeIdGenerator,
}

impl WebRepository {
    pub fn new(pool: AnyPool, id_generator: SnowflakeIdGenerator) -> Self {
        Self { pool, id_generator }
    }

    pub fn pool(&self) -> &AnyPool {
        &self.pool
    }

    pub fn id_generator(&self) -> &SnowflakeIdGenerator {
        &self.id_generator
    }
}
