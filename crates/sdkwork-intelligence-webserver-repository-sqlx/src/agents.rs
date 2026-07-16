use sdkwork_utils_rust::crypto::sha256_hash;
use sdkwork_webserver_contract::{
    AgentCertificateBundle, AgentHeartbeatRequest, AgentHeartbeatResponse, AgentNginxConfigBundle,
    AgentSyncResponse, WebServiceError, WebServiceResult,
};
use serde_json::{json, Value};
use sqlx::{any::AnyRow, Row};

use crate::support::{new_agent_token, now_rfc3339, sha256_hex, store_error};
use crate::WebRepository;

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
        let pattern = format!("%\"agentTokenHash\":\"{token_hash}\"%");
        let row = sqlx::query(
            "SELECT uuid, tenant_id, name, host
             FROM web_server
             WHERE metadata LIKE $1",
        )
        .bind(pattern)
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

        sqlx::query(
            "UPDATE web_server SET status = 1, metadata = $2, updated_at = $3, version = version + 1
             WHERE tenant_id = $1 AND uuid = $4",
        )
        .bind(agent.tenant_id)
        .bind(metadata)
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
        let nginx_configs = self
            .load_active_nginx_configs_for_tenant(agent.tenant_id)
            .await?;
        let certificate_rows = self
            .load_active_certificates_for_tenant(agent.tenant_id)
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
    ) -> WebServiceResult<Vec<AgentNginxConfigBundle>> {
        let rows = sqlx::query(
            "SELECT nc.uuid, nc.config_content, nc.version, s.uuid AS site_uuid
             FROM web_nginx_config nc
             INNER JOIN web_site s ON s.id = nc.site_id
             WHERE nc.tenant_id = $1 AND nc.is_active = 1 AND nc.status = 1 AND s.deleted_at IS NULL",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("load active nginx configs for agent sync", error))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            let site_uuid: String = row.try_get("site_uuid").map_err(|error| {
                WebServiceError::Internal(format!("agent sync nginx site_uuid: {error}"))
            })?;
            let domain = self
                .resolve_site_primary_hostname_repo(tenant_id, &site_uuid)
                .await?;
            let config_content: String = row.try_get("config_content").map_err(|error| {
                WebServiceError::Internal(format!("agent sync nginx content: {error}"))
            })?;
            items.push(AgentNginxConfigBundle {
                config_id: row.try_get("uuid").map_err(|error| {
                    WebServiceError::Internal(format!("agent sync nginx uuid: {error}"))
                })?,
                domain,
                fingerprint: sha256_hex(&config_content),
                config_content,
                version: row.try_get("version").map_err(|error| {
                    WebServiceError::Internal(format!("agent sync nginx version: {error}"))
                })?,
            });
        }
        Ok(items)
    }

    async fn load_active_certificates_for_tenant(
        &self,
        tenant_id: i64,
    ) -> WebServiceResult<Vec<(AgentCertificateBundle, String)>> {
        // 通过 web_certificate → web_domain → web_site JOIN 过滤，
        // 仅返回有效（未删除）站点的证书，避免向 agent 泄漏已下线站点的 TLS 私钥。
        let rows = sqlx::query(
            "SELECT c.uuid, c.cert_name, c.fingerprint, c.metadata
             FROM web_certificate c
             INNER JOIN web_domain d ON d.id = c.domain_id
             INNER JOIN web_site s ON s.id = d.site_id
             WHERE c.tenant_id = $1 AND c.status = 1 AND s.deleted_at IS NULL",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("load active certificates for agent sync", error))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            let metadata_raw: String = row.try_get("metadata").map_err(|error| {
                WebServiceError::Internal(format!("agent sync certificate metadata: {error}"))
            })?;
            let metadata: Value = serde_json::from_str(&metadata_raw).map_err(|error| {
                WebServiceError::Internal(format!("parse certificate metadata: {error}"))
            })?;
            let cert_pem = metadata
                .get("certPem")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            if cert_pem.is_empty() {
                continue;
            }
            let encrypted_private_key = metadata
                .get("encryptedPrivateKey")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    WebServiceError::Internal(
                        "active certificate missing encryptedPrivateKey metadata".to_string(),
                    )
                })?;

            items.push((
                AgentCertificateBundle {
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
                },
                encrypted_private_key.to_string(),
            ));
        }
        Ok(items)
    }
}

async fn merge_server_metadata(
    pool: &sqlx::AnyPool,
    server_uuid: &str,
    patch: &Value,
) -> Result<String, WebServiceError> {
    let row = sqlx::query("SELECT metadata FROM web_server WHERE uuid = $1")
        .bind(server_uuid)
        .fetch_optional(pool)
        .await
        .map_err(|error| store_error("load web_server metadata", error))?
        .ok_or_else(|| WebServiceError::not_found("server not found"))?;

    let existing_raw: String = row
        .try_get("metadata")
        .map_err(|error| WebServiceError::Internal(format!("read server metadata: {error}")))?;
    let mut existing: Value = serde_json::from_str(&existing_raw).unwrap_or_else(|_| json!({}));
    if let Some(object) = existing.as_object_mut() {
        if let Some(patch_object) = patch.as_object() {
            for (key, value) in patch_object {
                object.insert(key.clone(), value.clone());
            }
        }
    }
    Ok(existing.to_string())
}

fn map_authenticated_agent(row: &AnyRow) -> Result<AuthenticatedAgent, sqlx::Error> {
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
}
