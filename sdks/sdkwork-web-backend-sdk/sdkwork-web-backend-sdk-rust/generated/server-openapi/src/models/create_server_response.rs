use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateServerResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,

    #[serde(rename = "sshPort")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh_port: Option<i64>,

    /// 0=offline, 1=online
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<i64>,

    #[serde(rename = "lastHeartbeatAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_heartbeat_at: Option<String>,

    #[serde(rename = "createdAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// Bootstrap agent credential; returned once at registration.
    #[serde(rename = "agentToken")]
    pub agent_token: String,
}
