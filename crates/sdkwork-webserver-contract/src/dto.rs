use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SiteResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "siteType")]
    pub site_type: i32,
    pub status: i32,
    #[serde(rename = "runtimeConfig", skip_serializing_if = "Option::is_none")]
    pub runtime_config: Option<Value>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SitePage {
    pub items: Vec<SiteResponse>,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateSiteRequest {
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "siteType")]
    pub site_type: i32,
    #[serde(rename = "runtimeConfig", default)]
    pub runtime_config: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UpdateSiteRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "runtimeConfig", default)]
    pub runtime_config: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DomainResponse {
    pub id: String,
    pub hostname: String,
    #[serde(rename = "isPrimary")]
    pub is_primary: bool,
    #[serde(rename = "isVerified")]
    pub is_verified: bool,
    #[serde(rename = "sslEnabled")]
    pub ssl_enabled: bool,
    #[serde(rename = "sslProvider", skip_serializing_if = "Option::is_none")]
    pub ssl_provider: Option<String>,
    pub status: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DomainPage {
    pub items: Vec<DomainResponse>,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub total: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateDomainRequest {
    pub hostname: String,
    #[serde(rename = "isPrimary", default)]
    pub is_primary: bool,
    #[serde(rename = "sslEnabled", default = "default_true")]
    pub ssl_enabled: bool,
    #[serde(rename = "sslProvider", default)]
    pub ssl_provider: Option<String>,
}

fn default_true() -> bool {
    true
}

pub(crate) fn default_page() -> i32 {
    1
}

pub(crate) fn default_page_size() -> i32 {
    20
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainVerifyResponse {
    pub verified: bool,
    #[serde(rename = "verifyToken", skip_serializing_if = "Option::is_none")]
    pub verify_token: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DeploymentResponse {
    pub id: String,
    #[serde(rename = "siteId")]
    pub site_id: String,
    pub status: i32,
    #[serde(rename = "deployType")]
    pub deploy_type: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DeploymentPage {
    pub items: Vec<DeploymentResponse>,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CreateDeploymentRequest {
    #[serde(rename = "deployType", default = "default_deploy_type")]
    pub deploy_type: i32,
    #[serde(default)]
    pub environment: Option<String>,
    /// 客户端提供的幂等键。相同 (tenant_id, idempotency_key) 的请求会返回已创建的 deployment，
    /// 保证网络重试场景下不会产生重复部署记录。
    #[serde(rename = "idempotencyKey", default)]
    pub idempotency_key: Option<String>,
}

fn default_deploy_type() -> i32 {
    1
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EnvVariableResponse {
    pub id: String,
    pub key: String,
    pub value: String,
    pub environment: String,
    #[serde(rename = "isSecret")]
    pub is_secret: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EnvVariablePage {
    pub items: Vec<EnvVariableResponse>,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub total: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateEnvVariableRequest {
    pub key: String,
    pub value: String,
    #[serde(default = "default_environment")]
    pub environment: String,
    #[serde(rename = "isSecret", default)]
    pub is_secret: bool,
}

fn default_environment() -> String {
    "production".to_string()
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CertificateResponse {
    pub id: String,
    #[serde(rename = "certName")]
    pub cert_name: String,
    #[serde(rename = "certType", skip_serializing_if = "Option::is_none")]
    pub cert_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    #[serde(rename = "notBefore", skip_serializing_if = "Option::is_none")]
    pub not_before: Option<String>,
    #[serde(rename = "notAfter", skip_serializing_if = "Option::is_none")]
    pub not_after: Option<String>,
    #[serde(rename = "autoRenew", skip_serializing_if = "Option::is_none")]
    pub auto_renew: Option<bool>,
    pub status: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CertificatePage {
    pub items: Vec<CertificateResponse>,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub total: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateCertificateRequest {
    #[serde(rename = "domainId")]
    pub domain_id: String,
    #[serde(rename = "certType")]
    pub cert_type: i32,
    #[serde(rename = "autoRenew", default = "default_true")]
    pub auto_renew: bool,
}

#[derive(Clone, Debug)]
pub struct CertificateIssueUpdate {
    pub cert_name: String,
    pub cert_type: i32,
    pub issuer: String,
    pub subject: String,
    pub san_list: String,
    pub fingerprint: String,
    pub cert_path: String,
    pub key_path: String,
    pub chain_path: Option<String>,
    pub not_before: String,
    pub not_after: String,
    pub auto_renew: bool,
    pub cert_pem: String,
    pub chain_pem: Option<String>,
    pub encrypted_private_key: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub id: String,
    #[serde(rename = "checkType")]
    pub check_type: i32,
    pub url: String,
    pub status: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HealthCheckPage {
    pub items: Vec<HealthCheckResponse>,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub total: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateHealthCheckRequest {
    #[serde(rename = "checkType")]
    pub check_type: i32,
    pub url: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NginxConfigResponse {
    pub id: String,
    #[serde(rename = "siteId")]
    pub site_id: String,
    #[serde(rename = "configName")]
    pub config_name: String,
    #[serde(rename = "configType")]
    pub config_type: i32,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    pub status: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NginxConfigPage {
    pub items: Vec<NginxConfigResponse>,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ListNginxConfigsQuery {
    #[serde(default = "crate::dto::default_page")]
    pub page: i32,
    #[serde(default = "crate::dto::default_page_size")]
    pub page_size: i32,
    #[serde(rename = "siteId", default)]
    pub site_id: Option<String>,
    #[serde(rename = "configType", default)]
    pub config_type: Option<i32>,
    #[serde(rename = "isActive", default)]
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateNginxConfigRequest {
    #[serde(rename = "siteId")]
    pub site_id: String,
    #[serde(rename = "configName")]
    pub config_name: String,
    #[serde(rename = "configType")]
    pub config_type: i32,
    #[serde(rename = "configContent")]
    pub config_content: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UpdateNginxConfigRequest {
    #[serde(rename = "configName", default)]
    pub config_name: Option<String>,
    #[serde(rename = "configContent", default)]
    pub config_content: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NginxValidateResponse {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NginxReloadResponse {
    pub reloaded: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NginxStatusResponse {
    pub running: bool,
    #[serde(rename = "activeConfigs", with = "sdkwork_utils_rust::serde_int64")]
    pub active_configs: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ServerResponse {
    pub id: String,
    pub name: String,
    pub host: String,
    #[serde(rename = "tenantScopeHash")]
    pub tenant_scope_hash: String,
    #[serde(rename = "sshPort")]
    pub ssh_port: i32,
    pub status: i32,
    #[serde(rename = "lastHeartbeatAt", skip_serializing_if = "Option::is_none")]
    pub last_heartbeat_at: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateServerResponse {
    #[serde(flatten)]
    pub server: ServerResponse,
    #[serde(rename = "agentToken")]
    pub agent_token: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ServerPage {
    pub items: Vec<ServerResponse>,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub total: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub host: String,
    #[serde(rename = "tenantScopeHash")]
    pub tenant_scope_hash: String,
    #[serde(rename = "sshPort", default = "default_ssh_port")]
    pub ssh_port: i32,
}

fn default_ssh_port() -> i32 {
    22
}

#[derive(Clone, Debug)]
pub struct CertificateRenewalCandidate {
    pub tenant_id: i64,
    pub certificate_id: String,
    pub cert_type: i32,
    pub cert_name: String,
    pub hostname: String,
    pub auto_renew: bool,
    pub not_after: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CertificateRenewalCycleReport {
    pub scanned: usize,
    pub renewed: usize,
    pub failed: usize,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AgentHeartbeatRequest {
    #[serde(rename = "agentVersion", skip_serializing_if = "Option::is_none")]
    pub agent_version: Option<String>,
    #[serde(rename = "nginxEnabled", skip_serializing_if = "Option::is_none")]
    pub nginx_enabled: Option<bool>,
    #[serde(
        rename = "activeConfigs",
        with = "sdkwork_utils_rust::serde_int64::option",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub active_configs: Option<i64>,
    #[serde(rename = "lastSyncVersion", skip_serializing_if = "Option::is_none")]
    pub last_sync_version: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AgentHeartbeatResponse {
    #[serde(rename = "serverId")]
    pub server_id: String,
    pub status: i32,
    #[serde(rename = "acknowledgedAt")]
    pub acknowledged_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AgentSyncResponse {
    #[serde(rename = "serverId")]
    pub server_id: String,
    #[serde(rename = "syncVersion")]
    pub sync_version: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub unchanged: bool,
    #[serde(rename = "nginxConfigs")]
    pub nginx_configs: Vec<AgentNginxConfigBundle>,
    pub certificates: Vec<AgentCertificateBundle>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AgentNginxConfigBundle {
    #[serde(rename = "configId")]
    pub config_id: String,
    pub domain: String,
    #[serde(rename = "configContent")]
    pub config_content: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub fingerprint: String,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub version: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AgentCertificateBundle {
    #[serde(rename = "certificateId")]
    pub certificate_id: String,
    #[serde(rename = "certName")]
    pub cert_name: String,
    pub fingerprint: String,
    #[serde(rename = "fullchainPem")]
    pub fullchain_pem: String,
    #[serde(rename = "privkeyPem")]
    pub privkey_pem: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuditLogResponse {
    pub id: String,
    pub action: String,
    pub resource: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuditLogPage {
    pub items: Vec<AuditLogResponse>,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_totals_serialize_as_decimal_strings() {
        let page = SitePage {
            items: Vec::new(),
            total: 1_234_567_890_123,
            page: 1,
            page_size: 20,
        };
        let json = serde_json::to_value(&page).unwrap();
        assert_eq!(json["total"], serde_json::json!("1234567890123"));
        assert_eq!(json["page"], serde_json::json!(1));
    }

    #[test]
    fn agent_nginx_config_bundle_version_round_trips_as_string() {
        let bundle = AgentNginxConfigBundle {
            config_id: "cfg-1".into(),
            domain: "example.com".into(),
            config_content: "server {}".into(),
            fingerprint: "abc".into(),
            version: 9_876_543_210_987,
        };
        let json = serde_json::to_string(&bundle).unwrap();
        assert!(json.contains(r#""version":"9876543210987""#));
        let parsed: AgentNginxConfigBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, bundle.version);
    }

    #[test]
    fn agent_heartbeat_request_optional_int64_round_trips() {
        let request = AgentHeartbeatRequest {
            agent_version: Some("0.1".into()),
            nginx_enabled: Some(true),
            active_configs: Some(42),
            last_sync_version: Some("v1".into()),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains(r#""activeConfigs":"42""#));
        let parsed: AgentHeartbeatRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.active_configs, Some(42));
    }

    #[test]
    fn rejects_non_numeric_int64_string_input() {
        let json = r#"{"items":[],"total":"not-a-number","page":1,"pageSize":20}"#;
        let result: Result<SitePage, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
