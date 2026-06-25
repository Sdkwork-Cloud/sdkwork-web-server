//! Edge agent heartbeat and sync orchestration.

use sdkwork_webserver_contract::{
    AgentHeartbeatRequest, AgentHeartbeatResponse, AgentSyncResponse, WebServiceError,
    WebServiceResult,
};

use crate::WebService;

impl WebService {
    pub async fn agent_heartbeat(
        &self,
        token: &str,
        request: &AgentHeartbeatRequest,
    ) -> WebServiceResult<AgentHeartbeatResponse> {
        let (server_id, tenant_id) = self.repository.authenticate_agent_token(token).await?;
        self.repository
            .record_agent_heartbeat(&server_id, tenant_id, request)
            .await
    }

    pub async fn agent_sync(
        &self,
        token: &str,
        if_sync_version: Option<&str>,
    ) -> WebServiceResult<AgentSyncResponse> {
        let (server_id, tenant_id) = self.repository.authenticate_agent_token(token).await?;
        let (mut manifest, encrypted_private_keys) = self
            .repository
            .build_agent_sync_manifest(&server_id, tenant_id, if_sync_version)
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
