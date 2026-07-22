use std::collections::HashSet;
use std::sync::Arc;

use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_database_id::SnowflakeIdGenerator;
use sdkwork_database_sqlx::{create_any_pool_from_config, create_pool_from_config};
use sdkwork_intelligence_webserver_repository_sqlx::{PostgresWebRepository, SqliteWebRepository};
use sdkwork_intelligence_webserver_service::{
    AuditLogWrite, RuntimeAssignmentTarget, RuntimeAssignmentWrite, RuntimeObservationWrite,
    WebRepositoryPort,
};
use sdkwork_webserver_contract::{
    AgentHeartbeatRequest, CertificateIssueUpdate, CreateCertificateRequest,
    CreateDeploymentRequest, CreateDomainRequest, CreateEnvVariableRequest,
    CreateHealthCheckRequest, CreateNginxConfigRequest, CreateServerRequest, CreateSiteRequest,
    ListNginxConfigsQuery, ListSitesQuery, RuntimeObservationState, UpdateNginxConfigRequest,
    UpdateSiteRequest, WebServiceErrorKind, WebsiteRuntimeSetSnapshot,
};
use sdkwork_webserver_core::website_runtime::website_runtime_set_snapshot_sha256;
use sdkwork_webserver_database_host::bootstrap_web_database;
use sqlx::{AnyPool, Row};
use tempfile::TempDir;

const POSTGRES_TEST_URL_ENV: &str = "SDKWORK_WEB_POSTGRES_TEST_DATABASE_URL";
const TENANT_A: i64 = 410_001;
const TENANT_B: i64 = 410_002;

#[derive(Clone, Copy)]
enum TestEngine {
    Sqlite,
    Postgres,
}

struct TestContext {
    _directory: Option<TempDir>,
    pool: AnyPool,
    repository: Arc<dyn WebRepositoryPort>,
    engine: TestEngine,
}

#[tokio::test]
async fn sqlite_repository_transactions_tenants_idempotency_and_pagination_are_bounded() {
    let directory = TempDir::new().expect("create SQLite test directory");
    let database_path = directory.path().join("repository-parity.sqlite");
    let url = format!(
        "sqlite:///{}?mode=rwc",
        database_path.to_string_lossy().replace('\\', "/")
    );
    let context = prepare_database(
        DatabaseConfig {
            engine: DatabaseEngine::Sqlite,
            url,
            max_connections: 4,
            ..Default::default()
        },
        TestEngine::Sqlite,
        Some(directory),
    )
    .await;

    verify_repository_contract(&context).await;
    context.pool.close().await;
}

#[tokio::test]
#[ignore = "requires an explicitly configured disposable PostgreSQL database"]
async fn postgres_repository_transactions_tenants_idempotency_and_pagination_are_bounded() {
    let url = std::env::var(POSTGRES_TEST_URL_ENV).unwrap_or_else(|_| {
        panic!("set {POSTGRES_TEST_URL_ENV} to a disposable empty PostgreSQL database")
    });
    assert!(
        url.starts_with("postgres://") || url.starts_with("postgresql://"),
        "{POSTGRES_TEST_URL_ENV} must be a PostgreSQL URL"
    );
    let context = prepare_database(
        DatabaseConfig {
            engine: DatabaseEngine::Postgres,
            url,
            max_connections: 4,
            ..Default::default()
        },
        TestEngine::Postgres,
        None,
    )
    .await;

    verify_repository_contract(&context).await;
    context.pool.close().await;
}

async fn prepare_database(
    config: DatabaseConfig,
    engine: TestEngine,
    directory: Option<TempDir>,
) -> TestContext {
    let lifecycle_pool = create_pool_from_config(config.clone())
        .await
        .expect("create lifecycle pool");
    if let Some(postgres) = lifecycle_pool.as_postgres() {
        let existing_tables: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM information_schema.tables \
             WHERE table_schema = current_schema() AND table_type = 'BASE TABLE'",
        )
        .fetch_one(postgres)
        .await
        .expect("inspect disposable PostgreSQL schema");
        assert_eq!(
            existing_tables, 0,
            "refusing to run repository parity against a non-empty PostgreSQL schema"
        );
    }
    let lifecycle_host = bootstrap_web_database(lifecycle_pool)
        .await
        .expect("initialize Web database lifecycle");

    let database_engine = config.engine;
    let pool = create_any_pool_from_config(config)
        .await
        .expect("create repository AnyPool");
    let id_generator = SnowflakeIdGenerator::new(731).expect("create test Snowflake generator");
    let repository = match lifecycle_host.pool() {
        sdkwork_database_sqlx::DatabasePool::Postgres(typed_pool, _) => {
            Arc::new(PostgresWebRepository::new(
                typed_pool.clone(),
                database_engine,
                id_generator,
                [0x5a; 32],
            )) as Arc<dyn WebRepositoryPort>
        }
        sdkwork_database_sqlx::DatabasePool::Sqlite(typed_pool, _) => {
            Arc::new(SqliteWebRepository::new(
                typed_pool.clone(),
                database_engine,
                id_generator,
                [0x5a; 32],
            )) as Arc<dyn WebRepositoryPort>
        }
    };
    TestContext {
        _directory: directory,
        pool,
        repository,
        engine,
    }
}

async fn verify_repository_contract(context: &TestContext) {
    let repository = &context.repository;
    let mut sites = Vec::new();
    for index in 0..4 {
        sites.push(
            repository
                .create_site(
                    TENANT_A,
                    Some(31),
                    Some(91),
                    &CreateSiteRequest {
                        name: format!("Alpha Site {index}"),
                        slug: Some(format!("alpha-{index}")),
                        description: None,
                        site_type: 1,
                        runtime_config: None,
                    },
                )
                .await
                .expect("create tenant A site"),
        );
    }
    let tenant_b_site = repository
        .create_site(
            TENANT_B,
            None,
            None,
            &CreateSiteRequest {
                name: "Tenant B".to_owned(),
                slug: Some("alpha-0".to_owned()),
                description: None,
                site_type: 1,
                runtime_config: None,
            },
        )
        .await
        .expect("same slug is valid in another tenant");
    assert_ne!(sites[0].id, tenant_b_site.id);

    let duplicate_slug = repository
        .create_site(
            TENANT_A,
            None,
            None,
            &CreateSiteRequest {
                name: "Duplicate".to_owned(),
                slug: Some("alpha-0".to_owned()),
                description: None,
                site_type: 1,
                runtime_config: None,
            },
        )
        .await
        .expect_err("same tenant slug must conflict");
    assert_eq!(duplicate_slug.kind(), WebServiceErrorKind::Conflict);

    repository
        .retrieve_site(TENANT_A, &tenant_b_site.id)
        .await
        .expect_err("tenant A must not retrieve tenant B site");
    repository
        .retrieve_site(TENANT_B, &sites[0].id)
        .await
        .expect_err("tenant B must not retrieve tenant A site");

    let tie_sql = match context.engine {
        TestEngine::Sqlite => "UPDATE web_site SET updated_at = $1 WHERE tenant_id = $2",
        TestEngine::Postgres => {
            "UPDATE web_site SET updated_at = CAST($1 AS TIMESTAMPTZ) WHERE tenant_id = $2"
        }
    };
    sqlx::query(tie_sql)
        .bind("2026-01-01T00:00:00.000Z")
        .bind(TENANT_A)
        .execute(&context.pool)
        .await
        .expect("create deterministic pagination ties");
    let query = ListSitesQuery {
        page: 1,
        page_size: 2,
        status: Some(0),
        site_type: Some(1),
        keyword: Some(" alpha ".to_owned()),
    };
    let first_page = repository
        .list_sites(TENANT_A, &query)
        .await
        .expect("list first filtered page");
    let second_page = repository
        .list_sites(
            TENANT_A,
            &ListSitesQuery {
                page: 2,
                ..query.clone()
            },
        )
        .await
        .expect("list second filtered page");
    assert_eq!(first_page.total, 4);
    assert_eq!(first_page.items.len(), 2);
    assert_eq!(second_page.items.len(), 2);
    let first_ids: HashSet<_> = first_page.items.iter().map(|site| &site.id).collect();
    assert!(
        second_page
            .items
            .iter()
            .all(|site| !first_ids.contains(&site.id)),
        "stable pages must not overlap"
    );
    let expected: Vec<String> = sqlx::query(
        "SELECT uuid FROM web_site WHERE tenant_id = $1 ORDER BY updated_at DESC, id DESC",
    )
    .bind(TENANT_A)
    .fetch_all(&context.pool)
    .await
    .expect("load expected stable ordering")
    .into_iter()
    .map(|row| row.try_get("uuid").expect("site uuid"))
    .collect();
    let observed: Vec<_> = first_page
        .items
        .iter()
        .chain(&second_page.items)
        .map(|site| site.id.clone())
        .collect();
    assert_eq!(observed, expected);

    let deep_page = repository
        .list_sites(
            TENANT_A,
            &ListSitesQuery {
                page: i32::MAX,
                page_size: i32::MAX,
                status: None,
                site_type: None,
                keyword: None,
            },
        )
        .await
        .expect("deep page must remain a bounded SQL query");
    assert!(deep_page.items.is_empty());
    assert_eq!(deep_page.page_size, 100);

    verify_deployment_idempotency(repository.as_ref(), &sites[0].id, &sites[1].id).await;
    verify_rollback_atomicity(context, &sites[0].id).await;
    verify_public_repository_surface(context, &sites[0].id).await;
}

async fn verify_public_repository_surface(context: &TestContext, site_id: &str) {
    let repository = &context.repository;
    let updated_site = repository
        .update_site(
            TENANT_A,
            site_id,
            &UpdateSiteRequest {
                name: Some("Alpha Site Updated".to_string()),
                description: Some("dual-engine repository parity".to_string()),
                runtime_config: Some(serde_json::json!({
                    "workers": 4,
                    "features": {"http2": true, "https": true}
                })),
            },
        )
        .await
        .expect("update site JSON and timestamp fields");
    assert_eq!(updated_site.name, "Alpha Site Updated");
    assert_eq!(
        updated_site
            .runtime_config
            .as_ref()
            .and_then(|value| value.pointer("/features/https"))
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        repository
            .set_site_status(TENANT_A, site_id, 1)
            .await
            .expect("update site status timestamp")
            .status,
        1
    );

    let domain = repository
        .create_domain(
            TENANT_A,
            site_id,
            &CreateDomainRequest {
                hostname: "parity.example.test".to_string(),
                is_primary: true,
                ssl_enabled: true,
                ssl_provider: Some("acme".to_string()),
            },
        )
        .await
        .expect("create domain with transactional primary update");
    assert_eq!(
        repository
            .retrieve_domain(TENANT_A, site_id, &domain.id)
            .await
            .expect("retrieve domain")
            .hostname,
        "parity.example.test"
    );
    assert_eq!(
        repository
            .list_domains(TENANT_A, site_id, 1, 20)
            .await
            .expect("list domains")
            .total,
        1
    );
    assert!(
        repository
            .verify_domain(TENANT_A, site_id, &domain.id)
            .await
            .expect("verify domain timestamp")
            .verified
    );

    let public_env = repository
        .create_env_variable(
            TENANT_A,
            site_id,
            &CreateEnvVariableRequest {
                key: "PUBLIC_MODE".to_string(),
                value: "strict".to_string(),
                environment: "production".to_string(),
                is_secret: false,
            },
        )
        .await
        .expect("create public environment variable");
    assert_eq!(public_env.value, "strict");
    let secret_env = repository
        .create_env_variable(
            TENANT_A,
            site_id,
            &CreateEnvVariableRequest {
                key: "PRIVATE_TOKEN".to_string(),
                value: "test-only-secret".to_string(),
                environment: "production".to_string(),
                is_secret: true,
            },
        )
        .await
        .expect("create encrypted environment variable");
    assert_eq!(secret_env.value, "***");
    let env_page = repository
        .list_env_variables(TENANT_A, site_id, Some("production"))
        .await
        .expect("list environment variables");
    assert_eq!(env_page.total, 2);
    assert!(env_page
        .items
        .iter()
        .any(|item| item.key == "PRIVATE_TOKEN" && item.value == "***"));

    repository
        .create_health_check(
            TENANT_A,
            site_id,
            &CreateHealthCheckRequest {
                check_type: 1,
                url: "https://parity.example.test/healthz".to_string(),
            },
        )
        .await
        .expect("create health check timestamps");
    assert_eq!(
        repository
            .list_health_checks(TENANT_A, site_id)
            .await
            .expect("list health checks")
            .total,
        1
    );

    let nginx = repository
        .create_nginx_config(
            TENANT_A,
            &CreateNginxConfigRequest {
                site_id: site_id.to_string(),
                config_name: "parity.conf".to_string(),
                config_type: 1,
                config_content: "server { listen 443 ssl; }".to_string(),
            },
        )
        .await
        .expect("create nginx config timestamps");
    let nginx = repository
        .update_nginx_config(
            Some(TENANT_A),
            &nginx.id,
            &UpdateNginxConfigRequest {
                config_name: Some("parity-updated.conf".to_string()),
                config_content: Some("server { listen 443 ssl http2; }".to_string()),
            },
        )
        .await
        .expect("update nginx config timestamp");
    let repository_validation = repository
        .validate_nginx_config(Some(TENANT_A), &nginx.id)
        .await
        .expect("repository validation remains fail closed");
    assert!(!repository_validation.valid);
    assert!(repository_validation
        .message
        .as_deref()
        .is_some_and(|message| message.contains("edge runtime")));
    assert_eq!(
        repository
            .list_nginx_configs(
                Some(TENANT_A),
                &ListNginxConfigsQuery {
                    page: 1,
                    page_size: 20,
                    site_id: Some(site_id.to_string()),
                    config_type: Some(1),
                    is_active: None,
                },
            )
            .await
            .expect("list nginx configs")
            .total,
        1
    );
    assert_eq!(
        repository
            .list_nginx_configs(
                None,
                &ListNginxConfigsQuery {
                    page: 1,
                    page_size: 20,
                    site_id: None,
                    config_type: Some(1),
                    is_active: None,
                },
            )
            .await
            .expect("list nginx configs across tenants")
            .total,
        1
    );
    repository
        .web_nginx_config(None, &nginx.id)
        .await
        .expect("atomically activate nginx config through global backend scope");

    let (certificate_id, _) = repository
        .insert_certificate_pending(TENANT_A, &domain.id, 1, true)
        .await
        .expect("insert pending certificate timestamps");
    let certificate = repository
        .finalize_certificate(
            TENANT_A,
            &certificate_id,
            &CertificateIssueUpdate {
                cert_name: "parity.example.test".to_string(),
                cert_type: 1,
                issuer: "SDKWork Test CA".to_string(),
                subject: "CN=parity.example.test".to_string(),
                san_list: "parity.example.test".to_string(),
                fingerprint: "sha256:repository-parity".to_string(),
                cert_path: "/test/fullchain.pem".to_string(),
                key_path: "/test/privkey.pem".to_string(),
                chain_path: Some("/test/chain.pem".to_string()),
                not_before: "2026-01-01T00:00:00Z".to_string(),
                not_after: "2027-01-01T00:00:00Z".to_string(),
                auto_renew: true,
                cert_pem: "test-fullchain-pem".to_string(),
                chain_pem: Some("test-chain-pem".to_string()),
                encrypted_private_key: "test-encrypted-private-key".to_string(),
            },
        )
        .await
        .expect("finalize certificate JSON and timestamps");
    assert_eq!(certificate.status, 1);
    assert_eq!(
        repository
            .list_certificates(TENANT_A, 1, 20)
            .await
            .expect("list certificate timestamp projections")
            .total,
        1
    );
    assert!(repository
        .list_certificates_due_for_renewal(3650, 20)
        .await
        .expect("list renewal timestamp projections")
        .iter()
        .any(|candidate| candidate.certificate_id == certificate_id));
    assert!(repository
        .mark_certificate_renewing(TENANT_A, &certificate_id)
        .await
        .expect("mark certificate renewing timestamp"));
    repository
        .fail_certificate_renewal(TENANT_A, &certificate_id, "synthetic parity failure")
        .await
        .expect("merge certificate JSON metadata");
    let failed_certificate = repository
        .create_certificate(
            TENANT_A,
            &CreateCertificateRequest {
                domain_id: domain.id.clone(),
                cert_type: 1,
                auto_renew: false,
            },
        )
        .await
        .expect("create certificate through public wrapper");
    repository
        .fail_certificate(
            TENANT_A,
            &failed_certificate.id,
            "synthetic issuance failure",
        )
        .await
        .expect("write certificate failure JSON and timestamp");

    let server = repository
        .create_server(
            TENANT_A,
            &CreateServerRequest {
                name: "Parity Edge".to_string(),
                host: "192.0.2.44".to_string(),
                tenant_scope_hash: "a".repeat(64),
                ssh_port: 22,
            },
        )
        .await
        .expect("create server JSON and timestamps");
    let authenticated = repository
        .authenticate_agent_token(&server.agent_token)
        .await
        .expect("authenticate agent token from JSON metadata");
    assert_eq!(authenticated, (server.server.id.clone(), TENANT_A));
    verify_runtime_assignment_contract(context, &server.server.id).await;
    repository
        .record_agent_heartbeat(
            &server.server.id,
            TENANT_A,
            &AgentHeartbeatRequest {
                agent_version: Some("1.0.0-test".to_string()),
                nginx_enabled: Some(true),
                active_configs: Some(1),
                last_sync_version: None,
            },
        )
        .await
        .expect("merge heartbeat JSON and timestamp");
    assert!(repository
        .list_servers(TENANT_A, 1, 20)
        .await
        .expect("list server JSON and timestamp projections")
        .items
        .iter()
        .any(|item| item.id == server.server.id && item.last_heartbeat_at.is_some()));
    let (sync, encrypted_keys) = repository
        .build_agent_sync_manifest(&server.server.id, TENANT_A, None)
        .await
        .expect("build agent sync JSON projections");
    assert_eq!(sync.nginx_configs.len(), 1);
    assert_eq!(sync.certificates.len(), 1);
    assert_eq!(encrypted_keys, vec!["test-encrypted-private-key"]);
    verify_node_sync_database_bounds(context, &server.server.id, &nginx.id, &certificate_id).await;

    repository
        .insert_audit_log(AuditLogWrite {
            tenant_id: TENANT_A,
            organization_id: 31,
            operator_id: 91,
            action: "repository.parity",
            target_type: "site",
            target_id: None,
            target_uuid: Some(site_id),
        })
        .await
        .expect("insert audit timestamp");
    let audit_page = repository
        .list_audit_logs(Some(TENANT_A), 1, 20)
        .await
        .expect("list audit timestamp projections");
    assert_eq!(audit_page.total, 1);
    assert_eq!(audit_page.items[0].action, "repository.parity");

    repository
        .delete_domain(TENANT_A, site_id, &domain.id)
        .await
        .expect("soft-delete domain timestamps");
    repository
        .delete_site(TENANT_A, site_id, Some(91))
        .await
        .expect("soft-delete site timestamps");
    repository
        .retrieve_site(TENANT_A, site_id)
        .await
        .expect_err("soft-deleted site must not be retrievable");
}

async fn verify_runtime_assignment_contract(context: &TestContext, node_uuid: &str) {
    let repository = &context.repository;
    let target = repository
        .resolve_runtime_assignment_target(TENANT_A, false, node_uuid)
        .await
        .expect("resolve tenant-owned runtime target");
    assert_eq!(target.node_uuid, node_uuid);
    assert_eq!(target.tenant_scope_hash, "a".repeat(64));
    assert_eq!(
        repository
            .resolve_runtime_assignment_target(0, true, node_uuid)
            .await
            .expect("authorized service resolves target tenant")
            .tenant_id,
        TENANT_A
    );
    assert_eq!(
        repository
            .resolve_runtime_assignment_target(TENANT_B, false, node_uuid)
            .await
            .expect_err("another tenant cannot resolve the target")
            .kind(),
        WebServiceErrorKind::NotFound
    );

    let production_one = runtime_assignment_write(&target, "production", 1, "production-one");
    let first = repository
        .publish_runtime_assignment(production_one.clone())
        .await
        .expect("publish first production assignment");
    let replay = repository
        .publish_runtime_assignment(production_one.clone())
        .await
        .expect("same generation and hash are idempotent");
    assert_eq!(replay.assignment_uuid, first.assignment_uuid);

    let generation_conflict = repository
        .publish_runtime_assignment(runtime_assignment_write(
            &target,
            "production",
            1,
            "generation-conflict",
        ))
        .await
        .expect_err("same generation with another hash must conflict");
    assert_eq!(generation_conflict.kind(), WebServiceErrorKind::Conflict);

    let staging_one = runtime_assignment_write(&target, "staging", 1, "staging-one");
    repository
        .publish_runtime_assignment(staging_one.clone())
        .await
        .expect("environment generations are isolated");

    let initial = repository
        .retrieve_current_runtime_assignment(TENANT_A, node_uuid, "production", None, None)
        .await
        .expect("retrieve current production assignment");
    assert!(!initial.unchanged);
    assert_eq!(initial.assignment.generation, "1");
    assert!(initial.runtime_set.is_some());
    let unchanged = repository
        .retrieve_current_runtime_assignment(
            TENANT_A,
            node_uuid,
            "production",
            Some(&initial.assignment.generation),
            Some(&initial.assignment.snapshot_sha256),
        )
        .await
        .expect("conditionally retrieve current assignment");
    assert!(unchanged.unchanged);
    assert!(unchanged.runtime_set.is_none());
    let changed = repository
        .retrieve_current_runtime_assignment(
            TENANT_A,
            node_uuid,
            "production",
            Some(&initial.assignment.generation),
            Some(&"f".repeat(64)),
        )
        .await
        .expect("a mismatched condition returns the assignment body");
    assert!(!changed.unchanged);
    assert!(changed.runtime_set.is_some());
    assert_eq!(
        repository
            .retrieve_current_runtime_assignment(TENANT_B, node_uuid, "production", None, None,)
            .await
            .expect_err("current assignment is tenant scoped")
            .kind(),
        WebServiceErrorKind::NotFound
    );

    let production_two = runtime_assignment_write(&target, "production", 2, "production-two");
    let second = repository
        .publish_runtime_assignment(production_two.clone())
        .await
        .expect("publish next production generation");
    assert_eq!(second.generation, "2");
    assert_eq!(
        repository
            .publish_runtime_assignment(production_one)
            .await
            .expect_err("a lower generation must remain stale")
            .kind(),
        WebServiceErrorKind::Conflict
    );

    let active_first = runtime_observation_write(
        &production_two,
        RuntimeObservationState::Active,
        Some("1.0.0"),
        None,
        None,
    );
    assert_eq!(
        repository
            .create_runtime_observation(active_first)
            .await
            .expect_err("observations cannot start at ACTIVE")
            .kind(),
        WebServiceErrorKind::Conflict
    );

    let received_write = runtime_observation_write(
        &production_two,
        RuntimeObservationState::Received,
        Some("1.0.0"),
        None,
        None,
    );
    let received = repository
        .create_runtime_observation(received_write.clone())
        .await
        .expect("record RECEIVED");
    let received_replay = repository
        .create_runtime_observation(received_write.clone())
        .await
        .expect("identical observation is idempotent");
    assert_eq!(received_replay.observation_uuid, received.observation_uuid);
    let mut changed_received = received_write;
    changed_received.node_version = Some("1.0.1".to_owned());
    assert_eq!(
        repository
            .create_runtime_observation(changed_received)
            .await
            .expect_err("same state cannot be replayed with another payload")
            .kind(),
        WebServiceErrorKind::Conflict
    );
    assert_eq!(
        repository
            .create_runtime_observation(runtime_observation_write(
                &production_two,
                RuntimeObservationState::Staged,
                Some("1.0.0"),
                None,
                None,
            ))
            .await
            .expect_err("normal observation phases cannot be skipped")
            .kind(),
        WebServiceErrorKind::Conflict
    );

    for state in [
        RuntimeObservationState::Validated,
        RuntimeObservationState::Staged,
        RuntimeObservationState::Active,
    ] {
        repository
            .create_runtime_observation(runtime_observation_write(
                &production_two,
                state,
                Some("1.0.0"),
                None,
                None,
            ))
            .await
            .expect("advance observation state");
    }
    assert_eq!(
        repository
            .retrieve_current_runtime_assignment(
                TENANT_A,
                node_uuid,
                "production",
                Some(&production_two.generation.to_string()),
                Some(&production_two.snapshot_sha256),
            )
            .await
            .expect("retrieve activation checkpoint")
            .latest_observation_state,
        Some(RuntimeObservationState::Active)
    );
    assert_eq!(
        repository
            .create_runtime_observation(runtime_observation_write(
                &production_two,
                RuntimeObservationState::Rejected,
                Some("1.0.0"),
                Some("ACTIVATION_FAILED"),
                Some("must not replace ACTIVE"),
            ))
            .await
            .expect_err("terminal observations are immutable")
            .kind(),
        WebServiceErrorKind::Conflict
    );

    repository
        .create_runtime_observation(runtime_observation_write(
            &staging_one,
            RuntimeObservationState::Received,
            Some("1.0.0"),
            None,
            None,
        ))
        .await
        .expect("record staging RECEIVED");
    repository
        .create_runtime_observation(runtime_observation_write(
            &staging_one,
            RuntimeObservationState::Rejected,
            Some("1.0.0"),
            Some("VALIDATION_FAILED"),
            Some("synthetic parity rejection"),
        ))
        .await
        .expect("REJECTED may terminate any non-terminal phase");
    let mut generation_mismatch = runtime_observation_write(
        &staging_one,
        RuntimeObservationState::Rejected,
        Some("1.0.0"),
        Some("VALIDATION_FAILED"),
        Some("synthetic parity rejection"),
    );
    generation_mismatch.generation += 1;
    assert_eq!(
        repository
            .create_runtime_observation(generation_mismatch)
            .await
            .expect_err("observation generation must match assignment")
            .kind(),
        WebServiceErrorKind::Conflict
    );
}

fn runtime_assignment_write(
    target: &RuntimeAssignmentTarget,
    environment: &str,
    generation: u64,
    identity: &str,
) -> RuntimeAssignmentWrite {
    let snapshot_uuid = format!("snapshot-{generation}-{identity}");
    let mut value = serde_json::json!({
        "schemaVersion": "sdkwork.website-runtime-set.v1",
        "kind": "sdkwork.website-runtime-set.snapshot",
        "snapshotUuid": snapshot_uuid,
        "nodeUuid": target.node_uuid,
        "environment": environment,
        "generation": generation,
        "generatedAt": "2026-07-22T00:00:00Z",
        "compilerVersion": "repository-parity/1",
        "snapshotSha256": "0".repeat(64),
        "maximumSites": 8,
        "descriptors": []
    });
    let unsigned: WebsiteRuntimeSetSnapshot =
        serde_json::from_value(value.clone()).expect("parse unsigned runtime-set fixture");
    let snapshot_sha256 =
        website_runtime_set_snapshot_sha256(&unsigned).expect("hash runtime-set fixture");
    value["snapshotSha256"] = serde_json::Value::String(snapshot_sha256.clone());
    RuntimeAssignmentWrite {
        tenant_id: target.tenant_id,
        server_id: target.server_id,
        node_uuid: target.node_uuid.clone(),
        environment: environment.to_owned(),
        generation,
        snapshot_uuid,
        snapshot_sha256,
        runtime_set_json: serde_json::to_string(&value).expect("serialize runtime-set fixture"),
        runtime_set_bytes: serde_json::to_vec(&value)
            .expect("measure runtime-set fixture")
            .len(),
        assigned_by_subject: "repository-parity".to_owned(),
    }
}

fn runtime_observation_write(
    assignment: &RuntimeAssignmentWrite,
    state: RuntimeObservationState,
    node_version: Option<&str>,
    reason_code: Option<&str>,
    detail: Option<&str>,
) -> RuntimeObservationWrite {
    RuntimeObservationWrite {
        tenant_id: assignment.tenant_id,
        node_uuid: assignment.node_uuid.clone(),
        snapshot_uuid: assignment.snapshot_uuid.clone(),
        generation: assignment.generation,
        snapshot_sha256: assignment.snapshot_sha256.clone(),
        state,
        node_version: node_version.map(str::to_owned),
        reason_code: reason_code.map(str::to_owned),
        detail: detail.map(str::to_owned),
    }
}

async fn verify_node_sync_database_bounds(
    context: &TestContext,
    server_id: &str,
    nginx_config_id: &str,
    certificate_id: &str,
) {
    let original_config: String = sqlx::query_scalar(
        "SELECT config_content FROM web_nginx_config WHERE tenant_id = $1 AND uuid = $2",
    )
    .bind(TENANT_A)
    .bind(nginx_config_id)
    .fetch_one(&context.pool)
    .await
    .expect("read original node sync config");
    sqlx::query(
        "UPDATE web_nginx_config SET config_content = $1 WHERE tenant_id = $2 AND uuid = $3",
    )
    .bind("x".repeat(1024 * 1024 + 1))
    .bind(TENANT_A)
    .bind(nginx_config_id)
    .execute(&context.pool)
    .await
    .expect("install oversized node sync config");
    let oversized_config = context
        .repository
        .build_agent_sync_manifest(server_id, TENANT_A, None)
        .await
        .expect_err("oversized node sync config must fail closed");
    assert!(oversized_config
        .to_string()
        .contains("active nginx configuration exceeds"));
    sqlx::query(
        "UPDATE web_nginx_config SET config_content = $1 WHERE tenant_id = $2 AND uuid = $3",
    )
    .bind(original_config)
    .bind(TENANT_A)
    .bind(nginx_config_id)
    .execute(&context.pool)
    .await
    .expect("restore node sync config");

    let original_metadata: String = sqlx::query_scalar(
        "SELECT CAST(metadata AS TEXT) FROM web_certificate WHERE tenant_id = $1 AND uuid = $2",
    )
    .bind(TENANT_A)
    .bind(certificate_id)
    .fetch_one(&context.pool)
    .await
    .expect("read original node sync certificate metadata");
    let oversized_metadata = serde_json::json!({
        "certPem": "test-fullchain-pem",
        "encryptedPrivateKey": "test-encrypted-private-key",
        "padding": "x".repeat(2 * 1024 * 1024),
    })
    .to_string();
    let metadata_update = match context.engine {
        TestEngine::Sqlite => {
            "UPDATE web_certificate SET metadata = $1 WHERE tenant_id = $2 AND uuid = $3"
        }
        TestEngine::Postgres => {
            "UPDATE web_certificate SET metadata = CAST($1 AS JSONB) WHERE tenant_id = $2 AND uuid = $3"
        }
    };
    sqlx::query(metadata_update)
        .bind(oversized_metadata)
        .bind(TENANT_A)
        .bind(certificate_id)
        .execute(&context.pool)
        .await
        .expect("install oversized node sync certificate metadata");
    let oversized_certificate = context
        .repository
        .build_agent_sync_manifest(server_id, TENANT_A, None)
        .await
        .expect_err("oversized node sync certificate metadata must fail closed");
    assert!(oversized_certificate
        .to_string()
        .contains("active certificate metadata exceeds"));
    sqlx::query(metadata_update)
        .bind(original_metadata)
        .bind(TENANT_A)
        .bind(certificate_id)
        .execute(&context.pool)
        .await
        .expect("restore node sync certificate metadata");
}

async fn verify_deployment_idempotency(
    repository: &dyn WebRepositoryPort,
    first_site_id: &str,
    second_site_id: &str,
) {
    let request = CreateDeploymentRequest {
        deploy_type: 1,
        environment: Some("production".to_owned()),
        idempotency_key: Some("deploy-idempotency-1".to_owned()),
    };
    let first = repository
        .create_deployment(TENANT_A, first_site_id, Some(91), &request)
        .await
        .expect("create idempotent deployment");
    let repeated = repository
        .create_deployment(TENANT_A, first_site_id, Some(91), &request)
        .await
        .expect("repeat identical deployment");
    assert_eq!(repeated.id, first.id);

    let conflicting_input = repository
        .create_deployment(
            TENANT_A,
            first_site_id,
            Some(91),
            &CreateDeploymentRequest {
                deploy_type: 2,
                ..request.clone()
            },
        )
        .await
        .expect_err("same idempotency key with different input must conflict");
    assert_eq!(conflicting_input.kind(), WebServiceErrorKind::Conflict);
    let conflicting_site = repository
        .create_deployment(TENANT_A, second_site_id, Some(91), &request)
        .await
        .expect_err("tenant-wide idempotency key cannot move to another site");
    assert_eq!(conflicting_site.kind(), WebServiceErrorKind::Conflict);

    let concurrent_request = CreateDeploymentRequest {
        idempotency_key: Some("deploy-idempotency-race".to_owned()),
        ..request
    };
    let (left, right) = tokio::join!(
        repository.create_deployment(TENANT_A, first_site_id, Some(91), &concurrent_request),
        repository.create_deployment(TENANT_A, first_site_id, Some(91), &concurrent_request),
    );
    assert_eq!(
        left.expect("left concurrent idempotency result").id,
        right.expect("right concurrent idempotency result").id
    );
}

async fn verify_rollback_atomicity(context: &TestContext, site_id: &str) {
    let source = context
        .repository
        .create_deployment(
            TENANT_A,
            site_id,
            Some(91),
            &CreateDeploymentRequest {
                deploy_type: 1,
                environment: Some("production".to_owned()),
                idempotency_key: None,
            },
        )
        .await
        .expect("create rollback source");
    install_rollback_failure_trigger(&context.pool, context.engine).await;
    context
        .repository
        .rollback_deployment(TENANT_A, site_id, &source.id, Some(91))
        .await
        .expect_err("forced rollback-record failure must abort transaction");

    let status: i32 =
        sqlx::query_scalar("SELECT status FROM web_deployment WHERE tenant_id = $1 AND uuid = $2")
            .bind(TENANT_A)
            .bind(&source.id)
            .fetch_one(&context.pool)
            .await
            .expect("read rollback source status");
    assert_eq!(status, 0, "failed transaction must restore source status");
    let rollback_records: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM web_deployment WHERE tenant_id = $1 AND rollback_from IS NOT NULL",
    )
    .bind(TENANT_A)
    .fetch_one(&context.pool)
    .await
    .expect("count rollback records after forced failure");
    assert_eq!(rollback_records, 0);

    remove_rollback_failure_trigger(&context.pool, context.engine).await;
    let rollback = context
        .repository
        .rollback_deployment(TENANT_A, site_id, &source.id, Some(91))
        .await
        .expect("rollback succeeds after removing failure trigger");
    assert_eq!(rollback.site_id, site_id);
    let source_status: i32 =
        sqlx::query_scalar("SELECT status FROM web_deployment WHERE tenant_id = $1 AND uuid = $2")
            .bind(TENANT_A)
            .bind(&source.id)
            .fetch_one(&context.pool)
            .await
            .expect("read committed rollback source status");
    assert_eq!(source_status, 5);
}

async fn install_rollback_failure_trigger(pool: &AnyPool, engine: TestEngine) {
    match engine {
        TestEngine::Sqlite => {
            sqlx::query(
                "CREATE TRIGGER sdkwork_test_reject_rollback_insert
                 BEFORE INSERT ON web_deployment
                 WHEN NEW.rollback_from IS NOT NULL
                 BEGIN
                   SELECT RAISE(ABORT, 'forced rollback insert failure');
                 END",
            )
            .execute(pool)
            .await
            .expect("install SQLite rollback failure trigger");
        }
        TestEngine::Postgres => {
            sqlx::query(
                "CREATE FUNCTION sdkwork_test_reject_rollback_insert() RETURNS trigger AS $$
                 BEGIN
                   IF NEW.rollback_from IS NOT NULL THEN
                     RAISE EXCEPTION 'forced rollback insert failure';
                   END IF;
                   RETURN NEW;
                 END;
                 $$ LANGUAGE plpgsql",
            )
            .execute(pool)
            .await
            .expect("install PostgreSQL rollback failure function");
            sqlx::query(
                "CREATE TRIGGER sdkwork_test_reject_rollback_insert
                 BEFORE INSERT ON web_deployment
                 FOR EACH ROW EXECUTE FUNCTION sdkwork_test_reject_rollback_insert()",
            )
            .execute(pool)
            .await
            .expect("install PostgreSQL rollback failure trigger");
        }
    }
}

async fn remove_rollback_failure_trigger(pool: &AnyPool, engine: TestEngine) {
    match engine {
        TestEngine::Sqlite => {
            sqlx::query("DROP TRIGGER sdkwork_test_reject_rollback_insert")
                .execute(pool)
                .await
                .expect("remove SQLite rollback failure trigger");
        }
        TestEngine::Postgres => {
            sqlx::query("DROP TRIGGER sdkwork_test_reject_rollback_insert ON web_deployment")
                .execute(pool)
                .await
                .expect("remove PostgreSQL rollback failure trigger");
            sqlx::query("DROP FUNCTION sdkwork_test_reject_rollback_insert()")
                .execute(pool)
                .await
                .expect("remove PostgreSQL rollback failure function");
        }
    }
}
