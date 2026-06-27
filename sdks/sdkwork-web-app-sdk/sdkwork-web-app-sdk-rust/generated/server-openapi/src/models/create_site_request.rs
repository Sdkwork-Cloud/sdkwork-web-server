use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateSiteRequest {
    pub name: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "siteType")]
    pub site_type: i64,

    #[serde(rename = "runtimeConfig")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_config: Option<serde_json::Value>,
}
