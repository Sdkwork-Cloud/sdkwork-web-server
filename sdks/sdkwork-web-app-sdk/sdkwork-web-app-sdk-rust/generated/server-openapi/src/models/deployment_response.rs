use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DeploymentResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "siteId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,

    #[serde(rename = "deployType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deploy_type: Option<i64>,

    #[serde(rename = "versionTag")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_tag: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<i64>,

    #[serde(rename = "startedAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,

    #[serde(rename = "completedAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,

    /// Deployment duration in milliseconds as a string to avoid JavaScript precision loss.
    #[serde(rename = "durationMs")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<String>,

    #[serde(rename = "createdAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}
