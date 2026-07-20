use futures_util::TryStreamExt;
use sdkwork_database_config::DatabaseEngine;
use sdkwork_utils_rust::crypto::sha256_hash;
use sdkwork_webserver_contract::{
    AgentCertificateBundle, AgentHeartbeatRequest, AgentHeartbeatResponse, AgentNginxConfigBundle,
    AgentSyncResponse, WebServiceError, WebServiceResult,
};
use serde_json::{json, Value};
use super::{EnginePool, EngineRow, WebRepository};
use sqlx::Row;

use super::support::{
    instant_write_expression, json_from_row, json_write_expression, new_agent_token, now_rfc3339,
    sha256_hex, store_error,
};

const MAX_NODE_SYNC_ITEMS: usize = 2_048;
const MAX_NODE_SYNC_BUNDLE_BYTES: usize = 12 * 1024 * 1024;
const MAX_NODE_NGINX_CONFIG_BYTES: i64 = 1024 * 1024;
const MAX_NODE_CERTIFICATE_METADATA_BYTES: i64 = 2 * 1024 * 1024;

struct NodeSyncBudget {
    items: usize,
    serialized_bytes: usize,
    maximum_items: usize,
    maximum_serialized_bytes: usize,
}

impl NodeSyncBudget {
    fn new() -> Self {
        Self {
            items: 0,
            serialized_bytes: 0,
            maximum_items: MAX_NODE_SYNC_ITEMS,
            maximum_serialized_bytes: MAX_NODE_SYNC_BUNDLE_BYTES,
        }
    }

    fn reserve<T: serde::Serialize>(&mut self, item: &T) -> WebServiceResult<()> {
        self.reserve_with_additional_bytes(item, 0)
    }

    fn reserve_with_additional_bytes<T: serde::Serialize>(
        &mut self,
        item: &T,
        additional_bytes: usize,
    ) -> WebServiceResult<()> {
        if self.items >= self.maximum_items {
            return Err(WebServiceError::Internal(format!(
                "node sync manifest exceeds {} items",
                self.maximum_items
            )));
        }
        let item_bytes = serde_json::to_vec(item)
            .map_err(|error| WebServiceError::Internal(format!("encode node sync item: {error}")))?
            .len()
            .checked_add(additional_bytes)
            .ok_or_else(|| WebServiceError::Internal("node sync item byte overflow".to_string()))?;
        let serialized_bytes = self
            .serialized_bytes
            .checked_add(item_bytes)
            .ok_or_else(|| {
                WebServiceError::Internal("node sync byte budget overflow".to_string())
            })?;
        if serialized_bytes > self.maximum_serialized_bytes {
            return Err(WebServiceError::Internal(format!(
                "node sync manifest exceeds {} serialized bundle bytes",
                self.maximum_serialized_bytes
            )));
        }
        self.items += 1;
        self.serialized_bytes = serialized_bytes;
        Ok(())
    }

    #[cfg(test)]
    fn with_limits(maximum_items: usize, maximum_serialized_bytes: usize) -> Self {
        Self {
            items: 0,
            serialized_bytes: 0,
            maximum_items,
            maximum_serialized_bytes,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AuthenticatedAgent {
    pub server_uuid: String,
    pub tenant_id: i64,
}

pub(crate) fn hash_agent_token(token: &str) -> String {
    sha256_hash(token.as_bytes())
}

pub(crate) fn generate_agent_token() -> String {
    new_agent_token()
}

impl WebRepository {
    pub(super) async fn authenticate_agent_token_repo(
        &self,
        token: &str,
    ) -> WebServiceResult<AuthenticatedAgent> {
        let token_hash = hash_agent_token(token);
        let sql = match self.database_engine().await? {
            DatabaseEngine::Sqlite => {
                "SELECT uuid, tenant_id, name, host
                 FROM web_server
                 WHERE json_extract(metadata, '$.agentTokenHash') = $1"
            }
            DatabaseEngine::Postgres => {
                "SELECT uuid, tenant_id, name, host
                 FROM web_server
                 WHERE metadata ->> 'agentTokenHash' = $1"
            }
        };
        let row = sqlx::query(sql)
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| store_error("authenticate web_server agent token", error))?;

        let row = row.ok_or(WebServiceError::Forbidden)?;
        map_authenticated_agent(&row)
            .map_err(|error| WebServiceError::Internal(format!("map authenticated agent: {error}")))
    }

    pub(super) async fn record_agent_heartbeat_repo(
        &self,
        agent: &AuthenticatedAgent,
        request: &AgentHeartbeatRequest,
    ) -> WebServiceResult<AgentHeartbeatResponse> {
        let now = now_rfc3339();
        let metadata_patch = json!({
            "lastHeartbeatAt": now,
            "agentVersion": request.agent_version,
            "nginxEnabled": request.nginx_enabled,
            "activeConfigs": request.active_configs,
            "lastAppliedSyncVersion": request.last_sync_version,
        });

        let metadata =
            merge_server_metadata(&self.pool, &agent.server_uuid, &metadata_patch).await?;
        let engine = self.database_engine().await?;
        let metadata_expression = json_write_expression(engine, "$2");
        let now_expression = instant_write_expression(engine, "$3");
        let update_sql = format!(
            "UPDATE web_server SET status = 1, metadata = {metadata_expression},
                    updated_at = {now_expression}, version = version + 1
             WHERE tenant_id = $1 AND uuid = $4"
        );

        sqlx::query(&update_sql)
            .bind(agent.tenant_id)
            .bind(metadata.to_string())
            .bind(&now)
            .bind(&agent.server_uuid)
            .execute(&self.pool)
            .await
            .map_err(|error| store_error("record web_server heartbeat", error))?;

        Ok(AgentHeartbeatResponse {
            server_id: agent.server_uuid.clone(),
            status: 1,
            acknowledged_at: now,
        })
    }

    pub(super) async fn build_agent_sync_manifest_repo(
        &self,
        agent: &AuthenticatedAgent,
        if_sync_version: Option<&str>,
    ) -> WebServiceResult<(AgentSyncResponse, Vec<String>)> {
        let mut budget = NodeSyncBudget::new();
        let nginx_configs = self
            .load_active_nginx_configs_for_tenant(agent.tenant_id, &mut budget)
            .await?;
        let certificate_rows = self
            .load_active_certificates_for_tenant(agent.tenant_id, &mut budget)
            .await?;
        let mut encrypted_private_keys = Vec::with_capacity(certificate_rows.len());
        let mut certificates = Vec::with_capacity(certificate_rows.len());
        for (bundle, encrypted_private_key) in certificate_rows {
            encrypted_private_keys.push(encrypted_private_key);
            certificates.push(bundle);
        }
        let sync_version = compute_agent_sync_version(&nginx_configs, &certificates);

        if if_sync_version.is_some_and(|value| value == sync_version) {
            return Ok((
                AgentSyncResponse {
                    server_id: agent.server_uuid.clone(),
                    sync_version,
                    unchanged: true,
                    nginx_configs: Vec::new(),
                    certificates: Vec::new(),
                },
                Vec::new(),
            ));
        }

        Ok((
            AgentSyncResponse {
                server_id: agent.server_uuid.clone(),
                sync_version,
                unchanged: false,
                nginx_configs,
                certificates,
            },
            encrypted_private_keys,
        ))
    }

    async fn load_active_nginx_configs_for_tenant(
        &self,
        tenant_id: i64,
        budget: &mut NodeSyncBudget,
    ) -> WebServiceResult<Vec<AgentNginxConfigBundle>> {
        let content_size = match self.database_engine().await? {
            DatabaseEngine::Sqlite => "LENGTH(CAST(nc.config_content AS BLOB))",
            DatabaseEngine::Postgres => "OCTET_LENGTH(nc.config_content)",
        };
        let sql = format!(
            "SELECT nc.uuid,
                    CASE WHEN {content_size} <= {MAX_NODE_NGINX_CONFIG_BYTES}
                         THEN nc.config_content ELSE NULL END AS config_content,
                    {content_size} AS config_content_bytes,
                    nc.version,
                    (SELECT d.hostname FROM web_domain d
                     WHERE d.tenant_id = nc.tenant_id AND d.site_id = s.id
                       AND d.deleted_at IS NULL
                     ORDER BY d.is_primary DESC, d.created_at ASC
                     LIMIT 1) AS domain
             FROM web_nginx_config nc
             INNER JOIN web_site s ON s.id = nc.site_id
             WHERE nc.tenant_id = $1 AND nc.is_active = TRUE AND nc.status = 1
               AND s.deleted_at IS NULL
             ORDER BY nc.id ASC
             LIMIT {}",
            MAX_NODE_SYNC_ITEMS + 1
        );
        let mut rows = sqlx::query(&sql).bind(tenant_id).fetch(&self.pool);

        let mut items = Vec::new();
        while let Some(row) = rows
            .try_next()
            .await
            .map_err(|error| store_error("stream active nginx configs for agent sync", error))?
        {
            let content_bytes: i64 = row.try_get("config_content_bytes").map_err(|error| {
                WebServiceError::Internal(format!("agent sync nginx content bytes: {error}"))
            })?;
            if !(0..=MAX_NODE_NGINX_CONFIG_BYTES).contains(&content_bytes) {
                return Err(WebServiceError::Internal(format!(
                    "active nginx configuration exceeds {MAX_NODE_NGINX_CONFIG_BYTES} bytes"
                )));
            }
            let config_content: Option<String> =
                row.try_get("config_content").map_err(|error| {
                    WebServiceError::Internal(format!("agent sync nginx content: {error}"))
                })?;
            let config_content = config_content.ok_or_else(|| {
                WebServiceError::Internal("active nginx configuration is unavailable".to_string())
            })?;
            let domain: Option<String> = row.try_get("domain").map_err(|error| {
                WebServiceError::Internal(format!("agent sync nginx domain: {error}"))
            })?;
            let domain = domain.filter(|value| !value.is_empty()).ok_or_else(|| {
                WebServiceError::Internal(
                    "active nginx configuration has no deployable domain".to_string(),
                )
            })?;
            let item = AgentNginxConfigBundle {
                config_id: row.try_get("uuid").map_err(|error| {
                    WebServiceError::Internal(format!("agent sync nginx uuid: {error}"))
                })?,
                domain,
                fingerprint: sha256_hex(&config_content),
                config_content,
                version: row.try_get("version").map_err(|error| {
                    WebServiceError::Internal(format!("agent sync nginx version: {error}"))
                })?,
            };
            budget.reserve(&item)?;
            items.push(item);
        }
        Ok(items)
    }

    async fn load_active_certificates_for_tenant(
        &self,
        tenant_id: i64,
        budget: &mut NodeSyncBudget,
    ) -> WebServiceResult<Vec<(AgentCertificateBundle, String)>> {
        /*
        // 通过 web_certificate → web_domain → web_site JOIN 过滤，
        // 仅返回有效（未删除）站点的证书，避免向 agent 泄漏已下线站点的 TLS 私钥。
         */
        let metadata_size = match self.database_engine().await? {
            DatabaseEngine::Sqlite => "LENGTH(CAST(c.metadata AS BLOB))",
            DatabaseEngine::Postgres => "OCTET_LENGTH(CAST(c.metadata AS TEXT))",
        };
        let sql = format!(
            "SELECT c.uuid, c.cert_name, c.fingerprint,
                    CASE WHEN {metadata_size} <= {MAX_NODE_CERTIFICATE_METADATA_BYTES}
                         THEN CAST(c.metadata AS TEXT) ELSE NULL END AS metadata,
                    {metadata_size} AS metadata_bytes
             FROM web_certificate c
             INNER JOIN web_domain d ON d.id = c.domain_id
             INNER JOIN web_site s ON s.id = d.site_id
             WHERE c.tenant_id = $1 AND c.status = 1 AND s.deleted_at IS NULL
             ORDER BY c.id ASC
             LIMIT {}",
            MAX_NODE_SYNC_ITEMS + 1
        );
        let mut rows = sqlx::query(&sql).bind(tenant_id).fetch(&self.pool);

        let mut items = Vec::new();
        while let Some(row) = rows
            .try_next()
            .await
            .map_err(|error| store_error("stream active certificates for agent sync", error))?
        {
            let metadata_bytes: i64 = row.try_get("metadata_bytes").map_err(|error| {
                WebServiceError::Internal(format!("agent sync certificate metadata bytes: {error}"))
            })?;
            if !(0..=MAX_NODE_CERTIFICATE_METADATA_BYTES).contains(&metadata_bytes) {
                return Err(WebServiceError::Internal(format!(
                    "active certificate metadata exceeds {MAX_NODE_CERTIFICATE_METADATA_BYTES} bytes"
                )));
            }
            let metadata_raw: Option<String> = row.try_get("metadata").map_err(|error| {
                WebServiceError::Internal(format!("agent sync certificate metadata: {error}"))
            })?;
            let metadata_raw = metadata_raw.ok_or_else(|| {
                WebServiceError::Internal("active certificate metadata is unavailable".to_string())
            })?;
            let metadata: Value = serde_json::from_str(&metadata_raw).map_err(|error| {
                WebServiceError::Internal(format!(
                    "active certificate metadata is invalid: {error}"
                ))
            })?;
            let cert_pem = metadata
                .get("certPem")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    WebServiceError::Internal(
                        "active certificate missing certPem metadata".to_string(),
                    )
                })?
                .to_string();
            let encrypted_private_key = metadata
                .get("encryptedPrivateKey")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    WebServiceError::Internal(
                        "active certificate missing encryptedPrivateKey metadata".to_string(),
                    )
                })?;

            let item = AgentCertificateBundle {
                certificate_id: row.try_get("uuid").map_err(|error| {
                    WebServiceError::Internal(format!("agent sync certificate uuid: {error}"))
                })?,
                cert_name: row.try_get("cert_name").map_err(|error| {
                    WebServiceError::Internal(format!("agent sync certificate name: {error}"))
                })?,
                fingerprint: row.try_get("fingerprint").map_err(|error| {
                    WebServiceError::Internal(format!(
                        "agent sync certificate fingerprint: {error}"
                    ))
                })?,
                fullchain_pem: cert_pem,
                privkey_pem: String::new(),
            };
            budget.reserve_with_additional_bytes(&item, encrypted_private_key.len())?;
            items.push((item, encrypted_private_key.to_string()));
        }
        Ok(items)
    }
}

async fn merge_server_metadata(
    pool: &EnginePool,
    server_uuid: &str,
    patch: &Value,
) -> Result<Value, WebServiceError> {
    let row =
        sqlx::query("SELECT CAST(metadata AS TEXT) AS metadata FROM web_server WHERE uuid = $1")
            .bind(server_uuid)
            .fetch_optional(pool)
            .await
            .map_err(|error| store_error("load web_server metadata", error))?
            .ok_or_else(|| WebServiceError::not_found("server not found"))?;

    let mut existing = json_from_row(&row, "metadata")
        .map_err(|error| WebServiceError::Internal(format!("read server metadata: {error}")))?
        .unwrap_or_else(|| json!({}));
    if let Some(object) = existing.as_object_mut() {
        if let Some(patch_object) = patch.as_object() {
            for (key, value) in patch_object {
                object.insert(key.clone(), value.clone());
            }
        }
    }
    Ok(existing)
}

fn map_authenticated_agent(row: &EngineRow) -> Result<AuthenticatedAgent, sqlx::Error> {
    Ok(AuthenticatedAgent {
        server_uuid: row.try_get("uuid")?,
        tenant_id: row.try_get("tenant_id")?,
    })
}

pub(crate) fn parse_last_heartbeat_at(metadata_raw: &str) -> Option<String> {
    let metadata: Value = serde_json::from_str(metadata_raw).ok()?;
    metadata
        .get("lastHeartbeatAt")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

pub(crate) fn compute_agent_sync_version(
    nginx_configs: &[AgentNginxConfigBundle],
    certificates: &[AgentCertificateBundle],
) -> String {
    let mut parts = Vec::with_capacity(nginx_configs.len() + certificates.len());
    for config in nginx_configs {
        parts.push(format!(
            "n:{}:{}:{}",
            config.config_id, config.fingerprint, config.version
        ));
    }
    for certificate in certificates {
        parts.push(format!(
            "c:{}:{}",
            certificate.certificate_id, certificate.fingerprint
        ));
    }
    parts.sort_unstable();
    format!("sv1:{}", sha256_hash(parts.join("\n").as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_version_is_stable_for_same_manifest() {
        let nginx = vec![AgentNginxConfigBundle {
            config_id: "cfg-1".to_string(),
            domain: "example.com".to_string(),
            config_content: "server {}".to_string(),
            fingerprint: sha256_hex("server {}"),
            version: 2,
        }];
        let certs = vec![AgentCertificateBundle {
            certificate_id: "cert-1".to_string(),
            cert_name: "example.com".to_string(),
            fingerprint: "abc123".to_string(),
            fullchain_pem: String::new(),
            privkey_pem: String::new(),
        }];

        let first = compute_agent_sync_version(&nginx, &certs);
        let second = compute_agent_sync_version(&nginx, &certs);
        assert_eq!(first, second);
        assert!(first.starts_with("sv1:"));
    }

    #[test]
    fn sync_version_changes_when_certificate_fingerprint_changes() {
        let nginx = Vec::new();
        let certs_a = vec![AgentCertificateBundle {
            certificate_id: "cert-1".to_string(),
            cert_name: "example.com".to_string(),
            fingerprint: "abc123".to_string(),
            fullchain_pem: String::new(),
            privkey_pem: String::new(),
        }];
        let mut certs_b = certs_a.clone();
        certs_b[0].fingerprint = "def456".to_string();

        assert_ne!(
            compute_agent_sync_version(&nginx, &certs_a),
            compute_agent_sync_version(&nginx, &certs_b)
        );
    }

    #[test]
    fn sync_budget_rejects_item_and_serialized_byte_overflow() {
        let mut item_budget = NodeSyncBudget::with_limits(1, 1024);
        item_budget.reserve(&serde_json::json!({"id": 1})).unwrap();
        assert!(item_budget.reserve(&serde_json::json!({"id": 2})).is_err());

        let mut byte_budget = NodeSyncBudget::with_limits(2, 8);
        assert!(byte_budget
            .reserve(&serde_json::json!({"content": "too-large"}))
            .is_err());
    }
}
