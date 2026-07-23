-- Consolidated legacy baseline (SQLite adaptation)
-- Booleans as INTEGER, timestamps as ISO8601 TEXT, JSON as TEXT.

CREATE TABLE web_site (
    id              BIGINT       NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    organization_id INTEGER      NOT NULL DEFAULT 0,
    data_scope      INTEGER      NOT NULL DEFAULT 1,
    user_id         INTEGER,
    name            TEXT         NOT NULL,
    slug            TEXT         NOT NULL,
    description     TEXT,
    site_type       INTEGER      NOT NULL DEFAULT 1,
    status          INTEGER      NOT NULL DEFAULT 0,
    runtime_config  TEXT         NOT NULL DEFAULT '{}',
    metadata        TEXT         NOT NULL DEFAULT '{}',
    created_at      TEXT         NOT NULL,
    updated_at      TEXT         NOT NULL,
    version         INTEGER      NOT NULL DEFAULT 0,
    deleted_at      TEXT,
    deleted_by      INTEGER,
    PRIMARY KEY (id),
    CONSTRAINT chk_web_site_type CHECK (site_type BETWEEN 1 AND 6),
    CONSTRAINT chk_web_site_status CHECK (status BETWEEN 0 AND 3)
);

CREATE UNIQUE INDEX uk_web_site_uuid
    ON web_site (uuid);

CREATE UNIQUE INDEX uk_web_site_slug
    ON web_site (tenant_id, slug);

CREATE INDEX idx_web_site_tenant_status_updated
    ON web_site (tenant_id, organization_id, status, updated_at DESC);

CREATE INDEX idx_web_site_user_updated
    ON web_site (tenant_id, user_id, updated_at DESC);

CREATE INDEX idx_web_site_slug
    ON web_site (tenant_id, slug);

CREATE TABLE web_domain (
    id              BIGINT       NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    organization_id INTEGER      NOT NULL DEFAULT 0,
    site_id         INTEGER      NOT NULL,
    hostname        TEXT         NOT NULL,
    is_primary      INTEGER      NOT NULL DEFAULT 0,
    is_verified     INTEGER      NOT NULL DEFAULT 0,
    verify_token    TEXT,
    ssl_enabled     INTEGER      NOT NULL DEFAULT 0,
    ssl_provider    TEXT,
    redirect_target TEXT,
    status          INTEGER      NOT NULL DEFAULT 0,
    metadata        TEXT         NOT NULL DEFAULT '{}',
    created_at      TEXT         NOT NULL,
    updated_at      TEXT         NOT NULL,
    version         INTEGER      NOT NULL DEFAULT 0,
    deleted_at      TEXT,
    PRIMARY KEY (id),
    CONSTRAINT fk_web_domain_site FOREIGN KEY (site_id) REFERENCES web_site(id)
);

CREATE UNIQUE INDEX uk_web_domain_uuid
    ON web_domain (uuid);

CREATE UNIQUE INDEX uk_web_domain_hostname
    ON web_domain (hostname);

CREATE INDEX idx_web_domain_site
    ON web_domain (site_id);

CREATE INDEX idx_web_domain_tenant_status
    ON web_domain (tenant_id, status);

CREATE TABLE web_nginx_config (
    id              BIGINT       NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    site_id         INTEGER      NOT NULL,
    domain_id       INTEGER,
    config_type     INTEGER      NOT NULL DEFAULT 1,
    config_name     TEXT         NOT NULL,
    config_content  TEXT         NOT NULL,
    config_hash     TEXT         NOT NULL,
    is_active       INTEGER      NOT NULL DEFAULT 0,
    version_no      INTEGER      NOT NULL DEFAULT 1,
    deployed_at     TEXT,
    status          INTEGER      NOT NULL DEFAULT 0,
    metadata        TEXT         NOT NULL DEFAULT '{}',
    created_at      TEXT         NOT NULL,
    updated_at      TEXT         NOT NULL,
    version         INTEGER      NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT fk_web_nginx_config_site FOREIGN KEY (site_id) REFERENCES web_site(id)
);

CREATE UNIQUE INDEX uk_web_nginx_config_uuid
    ON web_nginx_config (uuid);

CREATE INDEX idx_web_nginx_config_site_active
    ON web_nginx_config (site_id, is_active);

CREATE INDEX idx_web_nginx_config_type_status
    ON web_nginx_config (config_type, status);

CREATE TABLE web_certificate (
    id              BIGINT       NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    domain_id       INTEGER,
    site_id         INTEGER,
    cert_name       TEXT         NOT NULL,
    cert_type       INTEGER      NOT NULL DEFAULT 1,
    issuer          TEXT,
    subject         TEXT,
    san_list        TEXT,
    fingerprint     TEXT,
    cert_path       TEXT,
    key_path        TEXT,
    chain_path      TEXT,
    not_before      TEXT,
    not_after       TEXT,
    auto_renew      INTEGER      NOT NULL DEFAULT 1,
    renewal_status  INTEGER      NOT NULL DEFAULT 0,
    status          INTEGER      NOT NULL DEFAULT 0,
    metadata        TEXT         NOT NULL DEFAULT '{}',
    created_at      TEXT         NOT NULL,
    updated_at      TEXT         NOT NULL,
    version         INTEGER      NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX uk_web_certificate_uuid
    ON web_certificate (uuid);

CREATE INDEX idx_web_certificate_domain
    ON web_certificate (domain_id);

CREATE INDEX idx_web_certificate_expiry
    ON web_certificate (not_after)
    WHERE status = 1;

CREATE INDEX idx_web_certificate_renewal
    ON web_certificate (renewal_status, not_after)
    WHERE auto_renew = true AND status = 1;

CREATE TABLE web_deployment (
    id              BIGINT       NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    organization_id INTEGER      NOT NULL DEFAULT 0,
    user_id         INTEGER,
    site_id         INTEGER      NOT NULL,
    deploy_type     INTEGER      NOT NULL DEFAULT 1,
    version_tag     TEXT,
    commit_hash     TEXT,
    source_ref      TEXT,
    build_log       TEXT,
    deploy_log      TEXT,
    artifact_path   TEXT,
    artifact_size   INTEGER,
    artifact_hash   TEXT,
    environment     TEXT         NOT NULL DEFAULT 'production',
    status          INTEGER      NOT NULL DEFAULT 0,
    started_at      TEXT,
    completed_at    TEXT,
    duration_ms     INTEGER,
    rollback_from   INTEGER,
    idempotency_key TEXT,
    request_id      TEXT,
    metadata        TEXT         NOT NULL DEFAULT '{}',
    created_at      TEXT         NOT NULL,
    updated_at      TEXT         NOT NULL,
    version         INTEGER      NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT fk_web_deployment_site FOREIGN KEY (site_id) REFERENCES web_site(id)
);

CREATE UNIQUE INDEX uk_web_deployment_uuid
    ON web_deployment (uuid);

CREATE UNIQUE INDEX uk_web_deployment_idempotency
    ON web_deployment (tenant_id, idempotency_key);

CREATE INDEX idx_web_deployment_site_created
    ON web_deployment (site_id, created_at DESC);

CREATE INDEX idx_web_deployment_tenant_status
    ON web_deployment (tenant_id, status, created_at DESC);

CREATE INDEX idx_web_deployment_status
    ON web_deployment (status)
    WHERE status IN (0, 1, 2);

CREATE TABLE web_env_variable (
    id              BIGINT       NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    site_id         INTEGER      NOT NULL,
    environment     TEXT         NOT NULL DEFAULT 'production',
    key             TEXT         NOT NULL,
    value_encrypted TEXT         NOT NULL,
    is_secret       INTEGER      NOT NULL DEFAULT 1,
    status          INTEGER      NOT NULL DEFAULT 1,
    created_at      TEXT         NOT NULL,
    updated_at      TEXT         NOT NULL,
    version         INTEGER      NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX uk_web_env_variable_uuid
    ON web_env_variable (uuid);

CREATE UNIQUE INDEX uk_web_env_variable_key
    ON web_env_variable (site_id, environment, key);

CREATE INDEX idx_web_env_variable_site_env
    ON web_env_variable (site_id, environment);

CREATE TABLE web_health_check (
    id              BIGINT       NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    site_id         INTEGER      NOT NULL,
    domain_id       INTEGER,
    check_type      INTEGER      NOT NULL DEFAULT 1,
    check_url       TEXT,
    check_interval  INTEGER      NOT NULL DEFAULT 60,
    timeout_ms      INTEGER      NOT NULL DEFAULT 5000,
    retry_count     INTEGER      NOT NULL DEFAULT 3,
    expected_status INTEGER,
    expected_body   TEXT,
    status          INTEGER      NOT NULL DEFAULT 1,
    created_at      TEXT         NOT NULL,
    updated_at      TEXT         NOT NULL,
    version         INTEGER      NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT fk_web_health_check_site FOREIGN KEY (site_id) REFERENCES web_site(id)
);

CREATE UNIQUE INDEX uk_web_health_check_uuid
    ON web_health_check (uuid);

CREATE INDEX idx_web_health_check_site
    ON web_health_check (site_id);

CREATE TABLE web_health_result (
    id              BIGINT       NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    health_check_id INTEGER      NOT NULL,
    site_id         INTEGER      NOT NULL,
    is_healthy      INTEGER      NOT NULL,
    response_ms     INTEGER,
    status_code     INTEGER,
    error_message   TEXT,
    checked_at      TEXT         NOT NULL,
    created_at      TEXT         NOT NULL,
    PRIMARY KEY (id)
);

CREATE INDEX idx_web_health_result_check_time
    ON web_health_result (health_check_id, checked_at DESC);

CREATE INDEX idx_web_health_result_site_time
    ON web_health_result (site_id, checked_at DESC);

CREATE TABLE web_audit_log (
    id              BIGINT       NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    organization_id INTEGER      NOT NULL DEFAULT 0,
    operator_id     INTEGER      NOT NULL,
    operator_type   TEXT         NOT NULL DEFAULT 'USER',
    action          TEXT         NOT NULL,
    target_type     TEXT         NOT NULL,
    target_id       INTEGER,
    target_uuid     TEXT,
    request_id      TEXT,
    ip_address      TEXT,
    user_agent      TEXT,
    changes         TEXT,
    metadata        TEXT         NOT NULL DEFAULT '{}',
    created_at      TEXT         NOT NULL,
    PRIMARY KEY (id)
);

CREATE INDEX idx_web_audit_log_target
    ON web_audit_log (target_type, target_id, created_at DESC);

CREATE INDEX idx_web_audit_log_operator
    ON web_audit_log (operator_id, created_at DESC);

CREATE INDEX idx_web_audit_log_tenant_action
    ON web_audit_log (tenant_id, action, created_at DESC);

CREATE TABLE web_server (
    id              BIGINT       NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    name            TEXT         NOT NULL,
    host            TEXT         NOT NULL,
    tenant_scope_hash TEXT       NOT NULL,
    ssh_port        INTEGER      NOT NULL DEFAULT 22,
    status          INTEGER      NOT NULL DEFAULT 0,
    metadata        TEXT         NOT NULL DEFAULT '{}',
    created_at      TEXT         NOT NULL,
    updated_at      TEXT         NOT NULL,
    version         INTEGER      NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_web_server_tenant_id UNIQUE (tenant_id, id),
    CONSTRAINT chk_web_server_tenant_scope_hash CHECK (
        length(tenant_scope_hash) = 64
        AND tenant_scope_hash NOT GLOB '*[^0-9a-f]*'
    )
);

CREATE UNIQUE INDEX uk_web_server_uuid
    ON web_server (uuid);

CREATE UNIQUE INDEX uk_web_server_host
    ON web_server (tenant_id, host);

CREATE INDEX idx_web_server_tenant_status
    ON web_server (tenant_id, status, updated_at DESC);

CREATE TABLE web_runtime_assignment (
    id                  BIGINT       NOT NULL,
    uuid                TEXT         NOT NULL,
    tenant_id           INTEGER      NOT NULL,
    server_id           BIGINT       NOT NULL,
    environment         TEXT         NOT NULL,
    generation          BIGINT       NOT NULL,
    snapshot_uuid       TEXT         NOT NULL,
    snapshot_sha256     TEXT         NOT NULL,
    runtime_set         TEXT         NOT NULL,
    runtime_set_bytes   BIGINT       NOT NULL,
    assigned_by_subject TEXT         NOT NULL,
    created_at          TEXT         NOT NULL,
    updated_at          TEXT         NOT NULL,
    version             INTEGER      NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_web_runtime_assignment_uuid UNIQUE (uuid),
    CONSTRAINT uk_web_runtime_assignment_tenant_id
        UNIQUE (tenant_id, id, server_id),
    CONSTRAINT uk_web_runtime_assignment_generation
        UNIQUE (tenant_id, server_id, environment, generation),
    CONSTRAINT uk_web_runtime_assignment_snapshot UNIQUE (snapshot_uuid),
    CONSTRAINT fk_web_runtime_assignment_server
        FOREIGN KEY (tenant_id, server_id) REFERENCES web_server (tenant_id, id),
    CONSTRAINT chk_web_runtime_assignment_environment
        CHECK (environment IN ('development', 'test', 'staging', 'production')),
    CONSTRAINT chk_web_runtime_assignment_generation
        CHECK (generation BETWEEN 1 AND 9007199254740991),
    CONSTRAINT chk_web_runtime_assignment_snapshot_sha256 CHECK (
        length(snapshot_sha256) = 64
        AND snapshot_sha256 NOT GLOB '*[^0-9a-f]*'
    ),
    CONSTRAINT chk_web_runtime_assignment_runtime_set
        CHECK (json_valid(runtime_set) AND json_type(runtime_set) = 'object'),
    CONSTRAINT chk_web_runtime_assignment_runtime_set_bytes
        CHECK (runtime_set_bytes BETWEEN 1 AND 67108864)
);

CREATE INDEX idx_web_runtime_assignment_current
    ON web_runtime_assignment (tenant_id, server_id, environment, generation DESC);

CREATE TABLE web_runtime_observation (
    id              BIGINT       NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL,
    assignment_id   BIGINT       NOT NULL,
    server_id       BIGINT       NOT NULL,
    state           TEXT         NOT NULL,
    node_version    TEXT,
    reason_code     TEXT,
    detail          TEXT,
    observed_at     TEXT         NOT NULL,
    created_at      TEXT         NOT NULL,
    updated_at      TEXT         NOT NULL,
    version         INTEGER      NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_web_runtime_observation_uuid UNIQUE (uuid),
    CONSTRAINT uk_web_runtime_observation_state
        UNIQUE (tenant_id, assignment_id, state),
    CONSTRAINT fk_web_runtime_observation_assignment
        FOREIGN KEY (tenant_id, assignment_id, server_id)
        REFERENCES web_runtime_assignment (tenant_id, id, server_id),
    CONSTRAINT chk_web_runtime_observation_state
        CHECK (state IN ('RECEIVED', 'VALIDATED', 'STAGED', 'ACTIVE', 'REJECTED')),
    CONSTRAINT chk_web_runtime_observation_node_version
        CHECK (node_version IS NULL OR length(node_version) BETWEEN 1 AND 64),
    CONSTRAINT chk_web_runtime_observation_reason_code
        CHECK (reason_code IS NULL OR length(reason_code) BETWEEN 1 AND 64),
    CONSTRAINT chk_web_runtime_observation_detail
        CHECK (detail IS NULL OR length(detail) BETWEEN 1 AND 512),
    CONSTRAINT chk_web_runtime_observation_reason CHECK (
        (state = 'REJECTED' AND reason_code IS NOT NULL)
        OR (state <> 'REJECTED' AND reason_code IS NULL AND detail IS NULL)
    )
);

CREATE INDEX idx_web_runtime_observation_assignment
    ON web_runtime_observation (tenant_id, assignment_id, id DESC);

CREATE INDEX idx_web_runtime_observation_node_time
    ON web_runtime_observation (tenant_id, server_id, observed_at DESC);
