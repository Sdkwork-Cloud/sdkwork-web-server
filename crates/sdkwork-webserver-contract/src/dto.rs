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
    pub total: i64,
    pub page: i32,
    #[serde(rename = "pageSize")]
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
    pub total: i64,
    pub page: i32,
    #[serde(rename = "pageSize")]
    pub page_size: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CreateDeploymentRequest {
    #[serde(rename = "deployType", default = "default_deploy_type")]
    pub deploy_type: i32,
    #[serde(default)]
    pub environment: Option<String>,
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
    pub total: i64,
    pub page: i32,
    #[serde(rename = "pageSize")]
    pub page_size: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ListNginxConfigsQuery {
    #[serde(default = "crate::dto::default_page")]
    pub page: i32,
    #[serde(default = "crate::dto::default_page_size", rename = "pageSize")]
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
    #[serde(rename = "activeConfigs")]
    pub active_configs: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ServerResponse {
    pub id: String,
    pub name: String,
    pub host: String,
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
    pub total: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub host: String,
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
    #[serde(rename = "activeConfigs", skip_serializing_if = "Option::is_none")]
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
    pub total: i64,
    pub page: i32,
    #[serde(rename = "pageSize")]
    pub page_size: i32,
}
