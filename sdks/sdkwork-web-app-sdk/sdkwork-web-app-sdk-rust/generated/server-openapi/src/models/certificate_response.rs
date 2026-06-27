use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CertificateResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "certName")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cert_name: Option<String>,

    #[serde(rename = "certType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cert_type: Option<i64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,

    #[serde(rename = "notBefore")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_before: Option<String>,

    #[serde(rename = "notAfter")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_after: Option<String>,

    #[serde(rename = "autoRenew")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_renew: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<i64>,

    #[serde(rename = "createdAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}
