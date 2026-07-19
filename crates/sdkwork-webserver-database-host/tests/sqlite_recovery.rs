use std::path::Path;
use std::sync::Arc;

use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_database_lifecycle::LifecycleOrchestrator;
use sdkwork_database_spi::DefaultDatabaseModule;
use sdkwork_database_sqlx::create_pool_from_config;
use tempfile::TempDir;

const MAX_BACKUP_BYTES: u64 = 64 * 1024 * 1024;
const CANARY_ID: i64 = 9_500_001;
const CANARY_TENANT_ID: i64 = 9_500;

fn application_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve application root")
}

fn sqlite_url(path: &Path, mode: &str) -> String {
    format!(
        "sqlite:///{}?mode={mode}",
        path.to_string_lossy().replace('\\', "/")
    )
}

#[tokio::test]
async fn sqlite_consistent_backup_restores_integrity_and_tenant_data() {
    let directory = TempDir::new().expect("create isolated SQLite recovery directory");
    let source_path = directory.path().join("source.sqlite");
    let backup_path = directory.path().join("backup.sqlite");
    let module = Arc::new(
        DefaultDatabaseModule::from_app_root(application_root()).expect("load Web database module"),
    );
    let source_pool = create_pool_from_config(DatabaseConfig {
        engine: DatabaseEngine::Sqlite,
        url: sqlite_url(&source_path, "rwc"),
        max_connections: 1,
        ..Default::default()
    })
    .await
    .expect("create source SQLite pool");
    let orchestrator = LifecycleOrchestrator::new(source_pool.clone(), module)
        .with_applied_by("sdkwork-webserver-recovery-test");
    orchestrator
        .init()
        .await
        .expect("initialize source SQLite database");

    let sqlite = source_pool.as_sqlite().expect("SQLite source pool");
    sqlx::query(
        "INSERT INTO web_site (\
            id, uuid, tenant_id, organization_id, data_scope, user_id, name, slug, \
            description, site_type, status, runtime_config, metadata, created_at, updated_at, \
            version, deleted_at, deleted_by\
         ) VALUES (?, ?, ?, 0, 1, NULL, ?, ?, NULL, 1, 1, '{}', '{}', ?, ?, 0, NULL, NULL)",
    )
    .bind(CANARY_ID)
    .bind("recovery-canary-uuid")
    .bind(CANARY_TENANT_ID)
    .bind("recovery-canary")
    .bind("recovery-canary")
    .bind("2026-07-19T00:00:00Z")
    .bind("2026-07-19T00:00:00Z")
    .execute(sqlite)
    .await
    .expect("insert SQLite recovery canary");

    sqlx::query("PRAGMA wal_checkpoint(FULL)")
        .execute(sqlite)
        .await
        .expect("checkpoint SQLite WAL before backup");
    let page_count: i64 = sqlx::query_scalar("PRAGMA page_count")
        .fetch_one(sqlite)
        .await
        .expect("read SQLite page count");
    let page_size: i64 = sqlx::query_scalar("PRAGMA page_size")
        .fetch_one(sqlite)
        .await
        .expect("read SQLite page size");
    let estimated_bytes = u64::try_from(page_count)
        .ok()
        .and_then(|count| {
            u64::try_from(page_size)
                .ok()
                .and_then(|size| count.checked_mul(size))
        })
        .expect("SQLite backup size estimate must be finite");
    assert!(
        estimated_bytes <= MAX_BACKUP_BYTES,
        "refusing an SQLite recovery fixture larger than {MAX_BACKUP_BYTES} bytes"
    );

    sqlx::query("VACUUM INTO ?")
        .bind(backup_path.to_string_lossy().as_ref())
        .execute(sqlite)
        .await
        .expect("create transactionally consistent SQLite backup");
    let backup_bytes = std::fs::metadata(&backup_path)
        .expect("read SQLite backup metadata")
        .len();
    assert!(backup_bytes > 0 && backup_bytes <= MAX_BACKUP_BYTES);

    sqlx::query("UPDATE web_site SET name = ? WHERE id = ?")
        .bind("source-mutated-after-backup")
        .bind(CANARY_ID)
        .execute(sqlite)
        .await
        .expect("mutate source after SQLite backup");
    source_pool.close().await;

    let restored_pool = create_pool_from_config(DatabaseConfig {
        engine: DatabaseEngine::Sqlite,
        url: sqlite_url(&backup_path, "rw"),
        max_connections: 1,
        ..Default::default()
    })
    .await
    .expect("open restored SQLite backup");
    let restored = restored_pool.as_sqlite().expect("restored SQLite pool");
    let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(restored)
        .await
        .expect("verify restored SQLite integrity");
    assert_eq!(integrity, "ok");
    let restored_name: String =
        sqlx::query_scalar("SELECT name FROM web_site WHERE id = ? AND tenant_id = ?")
            .bind(CANARY_ID)
            .bind(CANARY_TENANT_ID)
            .fetch_one(restored)
            .await
            .expect("read tenant-scoped canary from restored SQLite backup");
    assert_eq!(restored_name, "recovery-canary");
    let restored_tables: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table' AND name LIKE 'web_%'",
    )
    .fetch_one(restored)
    .await
    .expect("count restored Web tables");
    assert!(restored_tables >= 10, "restored Web schema is incomplete");
    restored_pool.close().await;
}
