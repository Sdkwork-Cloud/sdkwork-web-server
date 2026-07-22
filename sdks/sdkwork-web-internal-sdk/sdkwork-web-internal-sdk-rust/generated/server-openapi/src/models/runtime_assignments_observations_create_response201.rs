use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RuntimeAssignmentsObservationsCreateResponse201 {
    pub code: i64,

    pub message: String,

    pub data: serde_json::Value,

    #[serde(rename = "traceId")]
    pub trace_id: String,
}
