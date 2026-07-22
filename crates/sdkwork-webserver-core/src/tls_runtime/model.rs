use serde::{Deserialize, Serialize};

pub const TLS_RUNTIME_SNAPSHOT_KIND: &str = "sdkwork.tls-runtime.snapshot";
pub const TLS_RUNTIME_SCHEMA_VERSION: &str = "sdkwork.tls-runtime.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TlsAssignmentSnapshot {
    pub schema_version: String,
    pub kind: String,
    pub snapshot_uuid: String,
    pub node_uuid: String,
    pub generated_at: String,
    pub compiler_version: String,
    pub snapshot_sha256: String,
    pub assignments: Vec<TlsCertificateAssignment>,
    pub limits: TlsRuntimeLimits,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TlsCertificateAssignment {
    pub assignment_uuid: String,
    pub certificate_uuid: String,
    pub certificate_version: String,
    pub material_reference: String,
    pub expected_fingerprint_sha256: String,
    pub server_names: Vec<String>,
    pub not_before: String,
    pub not_after: String,
    pub policy: TlsRuntimePolicy,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TlsRuntimePolicy {
    pub minimum_version: TlsRuntimeVersion,
    pub maximum_version: TlsRuntimeVersion,
    pub alpn: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TlsRuntimeVersion {
    #[serde(rename = "TLS1_2")]
    Tls12,
    #[serde(rename = "TLS1_3")]
    Tls13,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TlsRuntimeLimits {
    pub maximum_assignments: usize,
    pub maximum_server_names_per_assignment: usize,
}
