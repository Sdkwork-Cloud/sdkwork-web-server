//! Edge agent heartbeat and sync orchestration.

use sdkwork_webserver_contract::{
    AgentHeartbeatRequest, AgentHeartbeatResponse, AgentSyncResponse, WebServiceError,
    WebServiceResult,
};

use crate::WebService;

impl WebService {
    /// Authenticates an agent bootstrap token and returns `(server_uuid, tenant_id)`.
    ///
    /// Called by `AgentTokenResolverDecorator` during the framework authentication stage
    /// (C8-C9) to resolve `X-SDKWork-Agent-Token` into a `WebRequestPrincipal`.
    pub async fn try_authenticate_agent_token(
        &self,
        token: &str,
    ) -> WebServiceResult<(String, i64)> {
        self.repository.authenticate_agent_token(token).await
    }

    /// Records an edge-agent heartbeat after the framework has already authenticated the token
    /// and resolved `server_id` + `tenant_id` via the `AgentTokenResolverDecorator` (C8-C9).
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
    /// and resolved `server_id` + `tenant_id` via the `AgentTokenResolverDecorator` (C8-C9).
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

        Ok(manifest)
    }
}
