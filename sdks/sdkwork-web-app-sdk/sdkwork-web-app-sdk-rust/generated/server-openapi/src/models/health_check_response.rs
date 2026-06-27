use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct HealthCheckResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "checkType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_type: Option<i64>,

    #[serde(rename = "checkUrl")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_url: Option<String>,

    #[serde(rename = "checkInterval")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_interval: Option<i64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<i64>,

    #[serde(rename = "createdAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}
