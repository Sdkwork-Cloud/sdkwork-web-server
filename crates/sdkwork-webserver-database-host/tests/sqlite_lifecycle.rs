use std::{path::PathBuf, sync::Arc};

use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_database_drift::DriftEngine;
use sdkwork_database_lifecycle::LifecycleOrchestrator;
use sdkwork_database_spi::{DefaultDatabaseModule, LocaleTag, SeedProfile};
use sdkwork_database_sqlx::create_pool_from_config;

fn application_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve application root")
}

#[tokio::test]
async fn sqlite_baseline_seed_and_drift_are_clean() {
    let module = Arc::new(
        DefaultDatabaseModule::from_app_root(application_root()).expect("load Web database module"),
    );
    let config = DatabaseConfig {
        engine: DatabaseEngine::Sqlite,
        url: "sqlite::memory:".to_owned(),
        max_connections: 1,
        ..Default::default()
    };
    let pool = create_pool_from_config(config)
        .await
        .expect("create SQLite pool");
    let orchestrator = LifecycleOrchestrator::new(pool.clone(), module.clone())
        .with_applied_by("sdkwork-webserver-test");

    orchestrator
        .init()
        .await
        .expect("initialize SQLite baseline");
    orchestrator
        .init()
        .await
        .expect("re-run idempotent SQLite baseline initialization");
    let applied = orchestrator
        .seed(&LocaleTag::zh_cn(), &SeedProfile::standard())
        .await
        .expect("seed SQLite database");
    assert_eq!(applied, 1);

    let reapplied = orchestrator
        .seed(&LocaleTag::zh_cn(), &SeedProfile::standard())
        .await
        .expect("re-run idempotent SQLite seed");
    assert_eq!(reapplied, 0);

    let report = DriftEngine::new(pool.clone(), module)
        .analyze()
        .await
        .expect("analyze SQLite drift");
    assert_eq!(report.status, "clean", "{:#?}", report.diffs);
    assert_eq!(report.summary.error, 0, "{:#?}", report.diffs);
    assert!(report.pending_migrations.is_empty());

    pool.close().await;
}
