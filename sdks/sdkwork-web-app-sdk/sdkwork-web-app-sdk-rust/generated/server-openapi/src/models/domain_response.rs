use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DomainResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,

    #[serde(rename = "isPrimary")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_primary: Option<bool>,

    #[serde(rename = "isVerified")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_verified: Option<bool>,

    #[serde(rename = "sslEnabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssl_enabled: Option<bool>,

    #[serde(rename = "sslProvider")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssl_provider: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<i64>,

    #[serde(rename = "createdAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}
