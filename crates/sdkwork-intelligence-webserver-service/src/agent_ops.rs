//! Web Node heartbeat and sync orchestration for the v3 Agent wire contract.

use sdkwork_webserver_contract::{
    AgentHeartbeatRequest, AgentHeartbeatResponse, AgentSyncResponse, WebServiceError,
    WebServiceResult,
};

use crate::WebService;

const MAX_NODE_SYNC_RESPONSE_BYTES: usize = 15 * 1024 * 1024;

impl WebService {
    /// Authenticates an agent bootstrap token and returns `(server_uuid, tenant_id)`.
    ///
    /// Called by `MachineCredentialResolverDecorator` during framework authentication
    /// (C8-C9) to resolve `X-SDKWork-Agent-Token` into a `WebRequestPrincipal`.
    pub async fn try_authenticate_agent_token(
        &self,
        token: &str,
    ) -> WebServiceResult<(String, i64)> {
        self.repository.authenticate_agent_token(token).await
    }

    /// Records an edge-agent heartbeat after the framework has already authenticated the token
    /// and resolved `server_id` + `tenant_id` via `MachineCredentialResolverDecorator` (C8-C9).
    pub async fn agent_heartbeat(
        &self,
        server_id: &str,
        tenant_id: i64,
        request: &AgentHeartbeatRequest,
    ) -> WebServiceResult<AgentHeartbeatResponse> {
        self.repository
            .record_agent_heartbeat(server_id, tenant_id, request)
            .await
    }

    /// Builds the agent sync manifest after the framework has already authenticated the token
    /// and resolved `server_id` + `tenant_id` via `MachineCredentialResolverDecorator` (C8-C9).
    pub async fn agent_sync(
        &self,
        server_id: &str,
        tenant_id: i64,
        if_sync_version: Option<&str>,
    ) -> WebServiceResult<AgentSyncResponse> {
        let (mut manifest, encrypted_private_keys) = self
            .repository
            .build_agent_sync_manifest(server_id, tenant_id, if_sync_version)
            .await?;

        if !manifest.unchanged {
            if manifest.certificates.len() != encrypted_private_keys.len() {
                return Err(WebServiceError::Internal(
                    "node sync certificate credential count mismatch".to_string(),
                ));
            }
            for (certificate, encrypted_private_key) in manifest
                .certificates
                .iter_mut()
                .zip(encrypted_private_keys.iter())
            {
                certificate.privkey_pem = self
                    .certificate_issuer
                    .decrypt_private_key(encrypted_private_key)
                    .map_err(|error| WebServiceError::Internal(error.to_string()))?;
            }
        }

        validate_node_sync_response_size(&manifest, MAX_NODE_SYNC_RESPONSE_BYTES)?;

        Ok(manifest)
    }
}

fn validate_node_sync_response_size(
    manifest: &AgentSyncResponse,
    maximum_bytes: usize,
) -> WebServiceResult<()> {
    let bytes = serde_json::to_vec(manifest)
        .map_err(|error| WebServiceError::Internal(format!("encode node sync response: {error}")))?
        .len();
    if bytes > maximum_bytes {
        return Err(WebServiceError::Internal(format!(
            "node sync response exceeds {maximum_bytes} bytes"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use sdkwork_webserver_contract::AgentSyncResponse;

    use super::validate_node_sync_response_size;

    #[test]
    fn node_sync_response_size_is_bounded_after_materialization() {
        let manifest = AgentSyncResponse {
            server_id: "node-1".to_string(),
            sync_version: "sv1:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            unchanged: false,
            nginx_configs: Vec::new(),
            certificates: Vec::new(),
        };
        let encoded = serde_json::to_vec(&manifest).unwrap();

        validate_node_sync_response_size(&manifest, encoded.len()).unwrap();
        assert!(validate_node_sync_response_size(&manifest, encoded.len() - 1).is_err());
    }
}
