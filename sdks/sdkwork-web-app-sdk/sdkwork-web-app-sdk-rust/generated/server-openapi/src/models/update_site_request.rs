use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateSiteRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "runtimeConfig")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_config: Option<std::collections::HashMap<String, serde_json::Value>>,
}
