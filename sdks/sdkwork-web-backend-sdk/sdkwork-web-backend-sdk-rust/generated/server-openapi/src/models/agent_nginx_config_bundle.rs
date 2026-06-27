use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AgentNginxConfigBundle {
    #[serde(rename = "configId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,

    #[serde(rename = "configContent")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_content: Option<String>,

    /// SHA-256 hex digest of configContent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,

    /// Config revision number as a string to avoid JavaScript precision loss.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
