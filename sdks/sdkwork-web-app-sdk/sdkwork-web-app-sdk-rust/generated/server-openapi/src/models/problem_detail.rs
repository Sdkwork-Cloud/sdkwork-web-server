use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProblemDetail {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<i64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,

    #[serde(rename = "requestId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}
