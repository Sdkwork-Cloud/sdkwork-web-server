use sdkwork_database_config::DatabaseEngine;
use sdkwork_database_id::SnowflakeIdGenerator;
use sqlx::{Database, Pool};

mod runtime;

pub use runtime::{bootstrap_web_runtime_from_env, WebRuntime};

/// AES-256 key used to protect environment-variable secrets at rest.
pub type SecretEncryptionKey = [u8; 32];

#[derive(Clone)]
pub struct TypedWebRepository<DB: Database> {
    pool: Pool<DB>,
    database_engine: DatabaseEngine,
    id_generator: SnowflakeIdGenerator,
    secret_key: SecretEncryptionKey,
}

impl<DB: Database> TypedWebRepository<DB> {
    pub fn new(
        pool: Pool<DB>,
        database_engine: DatabaseEngine,
        id_generator: SnowflakeIdGenerator,
        secret_key: SecretEncryptionKey,
    ) -> Self {
        Self {
            pool,
            database_engine,
            id_generator,
            secret_key,
        }
    }

    pub fn pool(&self) -> &Pool<DB> {
        &self.pool
    }

    pub fn id_generator(&self) -> &SnowflakeIdGenerator {
        &self.id_generator
    }

    pub fn secret_key(&self) -> &SecretEncryptionKey {
        &self.secret_key
    }

    pub(crate) async fn database_engine(
        &self,
    ) -> Result<DatabaseEngine, sdkwork_webserver_contract::WebServiceError> {
        Ok(self.database_engine)
    }
}

pub type PostgresWebRepository = TypedWebRepository<sqlx::Postgres>;
pub type SqliteWebRepository = TypedWebRepository<sqlx::Sqlite>;

macro_rules! repository_engine {
    ($module:ident, $database:ty, $pool:ty, $row:ty, $arguments:ty) => {
        mod $module {
            type WebRepository = crate::TypedWebRepository<$database>;
            type EnginePool = $pool;
            type EngineRow = $row;
            type EngineDatabase = $database;
            type EngineArguments<'q> = $arguments;

            mod support {
                include!("support.rs");
            }
            mod agents {
                include!("agents.rs");
            }
            mod audit {
                include!("audit.rs");
            }
            mod certificates {
                include!("certificates.rs");
            }
            mod deployments {
                include!("deployments.rs");
            }
            mod domains {
                include!("domains.rs");
            }
            mod domains_lookup {
                include!("domains_lookup.rs");
            }
            mod env_variables {
                include!("env_variables.rs");
            }
            mod health_checks {
                include!("health_checks.rs");
            }
            mod nginx_configs {
                include!("nginx_configs.rs");
            }
            mod runtime_assignments {
                include!("runtime_assignments.rs");
            }
            mod servers {
                include!("servers.rs");
            }
            mod sites {
                include!("sites.rs");
            }
            mod port {
                include!("port.rs");
            }
        }
    };
}

repository_engine!(
    postgres,
    sqlx::Postgres,
    sqlx::PgPool,
    sqlx::postgres::PgRow,
    sqlx::postgres::PgArguments
);
repository_engine!(
    sqlite,
    sqlx::Sqlite,
    sqlx::SqlitePool,
    sqlx::sqlite::SqliteRow,
    sqlx::sqlite::SqliteArguments<'q>
);
