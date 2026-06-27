use serde::{Deserialize, Serialize};

use crate::models::{AgentCertificateBundle, AgentNginxConfigBundle};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AgentSyncResponse {
    #[serde(rename = "serverId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,

    /// Stable SHA-256 fingerprint of active nginx configs and certificates for the tenant.
    #[serde(rename = "syncVersion")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync_version: Option<String>,

    /// True when ifSyncVersion matched syncVersion; bundles are omitted to save bandwidth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unchanged: Option<bool>,

    #[serde(rename = "nginxConfigs")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nginx_configs: Option<Vec<AgentNginxConfigBundle>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificates: Option<Vec<AgentCertificateBundle>>,
}
