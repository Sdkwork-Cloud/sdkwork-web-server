use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateServerRequest {
    pub name: String,

    pub host: String,

    #[serde(rename = "sshPort")]
    pub ssh_port: i64,

    #[serde(rename = "sshUser")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh_user: Option<String>,

    #[serde(rename = "sshKeyPath")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh_key_path: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
