use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AgentHeartbeatResponse {
    #[serde(rename = "serverId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<i64>,

    #[serde(rename = "acknowledgedAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acknowledged_at: Option<String>,
}
