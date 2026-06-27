use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AgentCertificateBundle {
    #[serde(rename = "certificateId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate_id: Option<String>,

    #[serde(rename = "certName")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cert_name: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,

    #[serde(rename = "fullchainPem")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fullchain_pem: Option<String>,

    #[serde(rename = "privkeyPem")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub privkey_pem: Option<String>,
}
