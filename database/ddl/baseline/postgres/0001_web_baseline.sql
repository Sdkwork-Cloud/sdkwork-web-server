-- Consolidated legacy baseline imported by bootstrap-database-module.mjs
-- Review and replace with contract-first migrations.

-- source: migrations/001_create_web_site.sql
-- Migration: 001_create_web_site
-- Description: ????????
-- Author: SDKWork Web Server
-- Date: 2026-06-14

CREATE TABLE web_site (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    data_scope      INTEGER      NOT NULL DEFAULT 1,
    user_id         BIGINT,
    name            VARCHAR(100) NOT NULL,
    slug            VARCHAR(100) NOT NULL,
    description     VARCHAR(500),
    site_type       INTEGER      NOT NULL DEFAULT 1,
    status          INTEGER      NOT NULL DEFAULT 0,
    runtime_config  JSONB        NOT NULL DEFAULT '{}',
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    deleted_at      TIMESTAMPTZ,
    deleted_by      BIGINT,
    PRIMARY KEY (id),
    CONSTRAINT uk_web_site_uuid UNIQUE (uuid),
    CONSTRAINT uk_web_site_slug UNIQUE (tenant_id, slug),
    CONSTRAINT chk_web_site_type CHECK (site_type BETWEEN 1 AND 6),
    CONSTRAINT chk_web_site_status CHECK (status BETWEEN 0 AND 3)
);

COMMENT ON TABLE web_site IS '????';
COMMENT ON COLUMN web_site.id IS '??ID??';
COMMENT ON COLUMN web_site.uuid IS '??????';
COMMENT ON COLUMN web_site.tenant_id IS '??ID';
COMMENT ON COLUMN web_site.organization_id IS '??ID????????;
COMMENT ON COLUMN web_site.data_scope IS '????????=????=????=??';
COMMENT ON COLUMN web_site.user_id IS '?????ID';
COMMENT ON COLUMN web_site.name IS '????';
COMMENT ON COLUMN web_site.slug IS 'URL??????????';
COMMENT ON COLUMN web_site.site_type IS '??????=???2=SPA??=Node??=PHP??=Python??=????;
COMMENT ON COLUMN web_site.status IS '???0=????=????=????=??';
COMMENT ON COLUMN web_site.runtime_config IS '?????JSON';
COMMENT ON COLUMN web_site.version IS '??????';

CREATE INDEX idx_web_site_tenant_status_updated
    ON web_site (tenant_id, organization_id, status, updated_at DESC);

CREATE INDEX idx_web_site_user_updated
    ON web_site (tenant_id, user_id, updated_at DESC);

CREATE INDEX idx_web_site_slug
    ON web_site (tenant_id, slug);

-- source: migrations/002_create_web_domain.sql
-- Migration: 002_create_web_domain
-- Description: ????????-- Author: SDKWork Web Server
-- Date: 2026-06-14

CREATE TABLE web_domain (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    site_id         BIGINT       NOT NULL,
    hostname        VARCHAR(255) NOT NULL,
    is_primary      BOOLEAN      NOT NULL DEFAULT false,
    is_verified     BOOLEAN      NOT NULL DEFAULT false,
    verify_token    VARCHAR(128),
    ssl_enabled     BOOLEAN      NOT NULL DEFAULT false,
    ssl_provider    VARCHAR(32),
    redirect_target VARCHAR(2000),
    status          INTEGER      NOT NULL DEFAULT 0,
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    deleted_at      TIMESTAMPTZ,
    PRIMARY KEY (id),
    CONSTRAINT uk_web_domain_uuid UNIQUE (uuid),
    CONSTRAINT uk_web_domain_hostname UNIQUE (hostname),
    CONSTRAINT fk_web_domain_site FOREIGN KEY (site_id) REFERENCES web_site(id)
);

COMMENT ON TABLE web_domain IS '??????;
COMMENT ON COLUMN web_domain.hostname IS '???????';
COMMENT ON COLUMN web_domain.is_primary IS '??????;
COMMENT ON COLUMN web_domain.is_verified IS '????????';
COMMENT ON COLUMN web_domain.ssl_enabled IS '????SSL';
COMMENT ON COLUMN web_domain.ssl_provider IS '??????letsencrypt, custom, none';
COMMENT ON COLUMN web_domain.status IS '???0=????1=????=??';

CREATE INDEX idx_web_domain_site
    ON web_domain (site_id);

CREATE INDEX idx_web_domain_tenant_status
    ON web_domain (tenant_id, status);

-- source: migrations/003_create_web_nginx_config.sql
-- Migration: 003_create_web_nginx_config
-- Description: ?? Nginx ??????-- Author: SDKWork Web Server
-- Date: 2026-06-14

CREATE TABLE web_nginx_config (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    site_id         BIGINT       NOT NULL,
    domain_id       BIGINT,
    config_type     INTEGER      NOT NULL DEFAULT 1,
    config_name     VARCHAR(200) NOT NULL,
    config_content  TEXT         NOT NULL,
    config_hash     VARCHAR(64)  NOT NULL,
    is_active       BOOLEAN      NOT NULL DEFAULT false,
    version_no      INTEGER      NOT NULL DEFAULT 1,
    deployed_at     TIMESTAMPTZ,
    status          INTEGER      NOT NULL DEFAULT 0,
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_web_nginx_config_uuid UNIQUE (uuid),
    CONSTRAINT fk_web_nginx_config_site FOREIGN KEY (site_id) REFERENCES web_site(id)
);

COMMENT ON TABLE web_nginx_config IS 'Nginx??????;
COMMENT ON COLUMN web_nginx_config.config_type IS '??????=????=????=SSL??=????;
COMMENT ON COLUMN web_nginx_config.config_content IS 'Nginx????';
COMMENT ON COLUMN web_nginx_config.config_hash IS '????SHA-256??';
COMMENT ON COLUMN web_nginx_config.is_active IS '??????????;
COMMENT ON COLUMN web_nginx_config.version_no IS '??????;
COMMENT ON COLUMN web_nginx_config.deployed_at IS 'deployed at';
COMMENT ON COLUMN web_nginx_config.status IS '???0=????=????=??';

CREATE INDEX idx_web_nginx_config_site_active
    ON web_nginx_config (site_id, is_active);

CREATE INDEX idx_web_nginx_config_type_status
    ON web_nginx_config (config_type, status);

-- source: migrations/004_create_web_certificate.sql
-- Migration: 004_create_web_certificate
-- Description: ?? SSL ????-- Author: SDKWork Web Server
-- Date: 2026-06-14

CREATE TABLE web_certificate (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    domain_id       BIGINT,
    site_id         BIGINT,
    cert_name       VARCHAR(200) NOT NULL,
    cert_type       INTEGER      NOT NULL DEFAULT 1,
    issuer          VARCHAR(200),
    subject         VARCHAR(500),
    san_list        TEXT,
    fingerprint     VARCHAR(128),
    cert_path       VARCHAR(500),
    key_path        VARCHAR(500),
    chain_path      VARCHAR(500),
    not_before      TIMESTAMPTZ,
    not_after       TIMESTAMPTZ,
    auto_renew      BOOLEAN      NOT NULL DEFAULT true,
    renewal_status  INTEGER      NOT NULL DEFAULT 0,
    status          INTEGER      NOT NULL DEFAULT 0,
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_web_certificate_uuid UNIQUE (uuid)
);

COMMENT ON TABLE web_certificate IS 'SSL????;
COMMENT ON COLUMN web_certificate.cert_type IS '??????=Let\'s Encrypt??=????3=????;
COMMENT ON COLUMN web_certificate.san_list IS 'Subject Alternative Names?????';
COMMENT ON COLUMN web_certificate.auto_renew IS '??????';
COMMENT ON COLUMN web_certificate.renewal_status IS '?????0=??1=????2=????3=??';
COMMENT ON COLUMN web_certificate.status IS '???0=????1=????=????=???';

CREATE INDEX idx_web_certificate_domain
    ON web_certificate (domain_id);

CREATE INDEX idx_web_certificate_expiry
    ON web_certificate (not_after)
    WHERE status = 1;

CREATE INDEX idx_web_certificate_renewal
    ON web_certificate (renewal_status, not_after)
    WHERE auto_renew = true AND status = 1;

-- source: migrations/005_create_web_deployment.sql
-- Migration: 005_create_web_deployment
-- Description: ????????-- Author: SDKWork Web Server
-- Date: 2026-06-14

CREATE TABLE web_deployment (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    user_id         BIGINT,
    site_id         BIGINT       NOT NULL,
    deploy_type     INTEGER      NOT NULL DEFAULT 1,
    version_tag     VARCHAR(100),
    commit_hash     VARCHAR(64),
    source_ref      VARCHAR(500),
    build_log       TEXT,
    deploy_log      TEXT,
    artifact_path   VARCHAR(500),
    artifact_size   BIGINT,
    artifact_hash   VARCHAR(64),
    environment     VARCHAR(32)  NOT NULL DEFAULT 'production',
    status          INTEGER      NOT NULL DEFAULT 0,
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    duration_ms     BIGINT,
    rollback_from   BIGINT,
    idempotency_key VARCHAR(200),
    request_id      VARCHAR(128),
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_web_deployment_uuid UNIQUE (uuid),
    CONSTRAINT uk_web_deployment_idempotency UNIQUE (tenant_id, idempotency_key),
    CONSTRAINT fk_web_deployment_site FOREIGN KEY (site_id) REFERENCES web_site(id)
);

COMMENT ON TABLE web_deployment IS '??????;
COMMENT ON COLUMN web_deployment.deploy_type IS '??????=????=Git??=CI/CD??=API';
COMMENT ON COLUMN web_deployment.status IS '???0=????1=????2=????3=????=????=????;
COMMENT ON COLUMN web_deployment.duration_ms IS '????????';
COMMENT ON COLUMN web_deployment.rollback_from IS '??????ID';
COMMENT ON COLUMN web_deployment.idempotency_key IS '?????????';

CREATE INDEX idx_web_deployment_site_created
    ON web_deployment (site_id, created_at DESC);

CREATE INDEX idx_web_deployment_tenant_status
    ON web_deployment (tenant_id, status, created_at DESC);

CREATE INDEX idx_web_deployment_status
    ON web_deployment (status)
    WHERE status IN (0, 1, 2);

-- source: migrations/006_create_web_env_variable.sql
-- Migration: 006_create_web_env_variable
-- Description: ????????-- Author: SDKWork Web Server
-- Date: 2026-06-14

CREATE TABLE web_env_variable (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    site_id         BIGINT       NOT NULL,
    environment     VARCHAR(32)  NOT NULL DEFAULT 'production',
    key             VARCHAR(200) NOT NULL,
    value_encrypted TEXT         NOT NULL,
    is_secret       BOOLEAN      NOT NULL DEFAULT true,
    status          INTEGER      NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_web_env_variable_uuid UNIQUE (uuid),
    CONSTRAINT uk_web_env_variable_key UNIQUE (site_id, environment, key)
);

COMMENT ON TABLE web_env_variable IS '??????;
COMMENT ON COLUMN web_env_variable.key IS '????;
COMMENT ON COLUMN web_env_variable.value_encrypted IS '?????????;
COMMENT ON COLUMN web_env_variable.is_secret IS '????????;
COMMENT ON COLUMN web_env_variable.environment IS '?????;

CREATE INDEX idx_web_env_variable_site_env
    ON web_env_variable (site_id, environment);

-- source: migrations/007_create_web_health_check.sql
-- Migration: 007_create_web_health_check
-- Description: ?????????
-- Author: SDKWork Web Server
-- Date: 2026-06-14

CREATE TABLE web_health_check (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    site_id         BIGINT       NOT NULL,
    domain_id       BIGINT,
    check_type      INTEGER      NOT NULL DEFAULT 1,
    check_url       VARCHAR(2000),
    check_interval  INTEGER      NOT NULL DEFAULT 60,
    timeout_ms      INTEGER      NOT NULL DEFAULT 5000,
    retry_count     INTEGER      NOT NULL DEFAULT 3,
    expected_status INTEGER,
    expected_body   VARCHAR(500),
    status          INTEGER      NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_web_health_check_uuid UNIQUE (uuid),
    CONSTRAINT fk_web_health_check_site FOREIGN KEY (site_id) REFERENCES web_site(id)
);

COMMENT ON TABLE web_health_check IS '???????';
COMMENT ON COLUMN web_health_check.check_type IS '?????1=HTTP??=TCP??=Ping';
COMMENT ON COLUMN web_health_check.check_interval IS '???????';
COMMENT ON COLUMN web_health_check.timeout_ms IS '????????';
COMMENT ON COLUMN web_health_check.retry_count IS '????';

CREATE INDEX idx_web_health_check_site
    ON web_health_check (site_id);

-- source: migrations/008_create_web_health_result.sql
-- Migration: 008_create_web_health_result
-- Description: ?????????
-- Author: SDKWork Web Server
-- Date: 2026-06-14

CREATE TABLE web_health_result (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    health_check_id BIGINT       NOT NULL,
    site_id         BIGINT       NOT NULL,
    is_healthy      BOOLEAN      NOT NULL,
    response_ms     INTEGER,
    status_code     INTEGER,
    error_message   VARCHAR(1000),
    checked_at      TIMESTAMPTZ  NOT NULL,
    created_at      TIMESTAMPTZ  NOT NULL,
    PRIMARY KEY (id)
);

COMMENT ON TABLE web_health_result IS '???????';
COMMENT ON COLUMN web_health_result.is_healthy IS '????';
COMMENT ON COLUMN web_health_result.response_ms IS '????????';
COMMENT ON COLUMN web_health_result.status_code IS 'HTTP???';
COMMENT ON COLUMN web_health_result.checked_at IS '?????;

CREATE INDEX idx_web_health_result_check_time
    ON web_health_result (health_check_id, checked_at DESC);

CREATE INDEX idx_web_health_result_site_time
    ON web_health_result (site_id, checked_at DESC);

-- source: migrations/009_create_web_audit_log.sql
-- Migration: 009_create_web_audit_log
-- Description: ??????????-- Author: SDKWork Web Server
-- Date: 2026-06-14

CREATE TABLE web_audit_log (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    operator_id     BIGINT       NOT NULL,
    operator_type   VARCHAR(32)  NOT NULL DEFAULT 'USER',
    action          VARCHAR(100) NOT NULL,
    target_type     VARCHAR(100) NOT NULL,
    target_id       BIGINT,
    target_uuid     VARCHAR(64),
    request_id      VARCHAR(128),
    ip_address      VARCHAR(45),
    user_agent      VARCHAR(500),
    changes         JSONB,
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL,
    PRIMARY KEY (id)
);

COMMENT ON TABLE web_audit_log IS '????????;
COMMENT ON COLUMN web_audit_log.operator_id IS '???ID';
COMMENT ON COLUMN web_audit_log.operator_type IS '??????USER, SYSTEM, ADMIN, JOB, SERVICE';
COMMENT ON COLUMN web_audit_log.action IS '????';
COMMENT ON COLUMN web_audit_log.target_type IS '??????';
COMMENT ON COLUMN web_audit_log.target_id IS '????ID';
COMMENT ON COLUMN web_audit_log.changes IS '????JSON?{"field": {"old": x, "new": y}}';

CREATE INDEX idx_web_audit_log_target
    ON web_audit_log (target_type, target_id, created_at DESC);

CREATE INDEX idx_web_audit_log_operator
    ON web_audit_log (operator_id, created_at DESC);

CREATE INDEX idx_web_audit_log_tenant_action
    ON web_audit_log (tenant_id, action, created_at DESC);

-- source: migrations/010_create_web_server.sql
-- Migration: 010_create_web_server
-- Description: ??????????
-- Author: SDKWork Web Server
-- Date: 2026-06-23

CREATE TABLE web_server (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    name            VARCHAR(200) NOT NULL,
    host            VARCHAR(255) NOT NULL,
    ssh_port        INTEGER      NOT NULL DEFAULT 22,
    status          INTEGER      NOT NULL DEFAULT 0,
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_web_server_uuid UNIQUE (uuid),
    CONSTRAINT uk_web_server_host UNIQUE (tenant_id, host)
);

COMMENT ON TABLE web_server IS '????????';
COMMENT ON COLUMN web_server.status IS '???0=????1=????=????=????;

CREATE INDEX idx_web_server_tenant_status
    ON web_server (tenant_id, status, updated_at DESC);

