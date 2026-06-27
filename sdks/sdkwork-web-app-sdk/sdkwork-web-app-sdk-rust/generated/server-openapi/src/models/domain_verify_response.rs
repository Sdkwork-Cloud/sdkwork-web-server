use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DomainVerifyResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}
