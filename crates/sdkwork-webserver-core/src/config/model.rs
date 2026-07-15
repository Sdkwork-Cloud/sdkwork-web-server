use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

fn default_nginx_profile() -> String {
    "http-core-v1".to_owned()
}

fn default_unknown_directive_policy() -> String {
    "error".to_owned()
}

fn default_max_request_body_bytes() -> u64 {
    10 * 1024 * 1024
}

fn default_request_timeout_ms() -> u64 {
    30_000
}

fn default_drain_timeout_ms() -> u64 {
    30_000
}

fn default_max_connections() -> usize {
    10_000
}

fn default_index_files() -> Vec<String> {
    vec!["index.html".to_owned()]
}

fn default_content_type() -> String {
    "text/plain; charset=utf-8".to_owned()
}

fn default_connect_timeout_ms() -> u64 {
    5_000
}

fn default_max_idle_connections() -> usize {
    128
}

fn default_weight() -> u16 {
    1
}

fn default_resolver_timeout_ms() -> u64 {
    2_000
}

fn default_maximum_answers() -> usize {
    16
}

fn default_access_log() -> bool {
    true
}

fn default_tls_minimum() -> TlsVersion {
    TlsVersion::Tls12
}

fn default_tls_maximum() -> TlsVersion {
    TlsVersion::Tls13
}

fn default_alpn() -> Vec<String> {
    vec!["h2".to_owned(), "http/1.1".to_owned()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WebServerAppConfig {
    pub schema_version: u32,
    pub kind: String,
    pub app_key: String,
    #[serde(default)]
    pub compatibility: CompatibilityConfig,
    #[serde(default)]
    pub limits: WebServerLimits,
    pub listeners: Vec<ListenerConfig>,
    #[serde(default)]
    pub certificates: Vec<CertificateConfig>,
    #[serde(default)]
    pub tls_policies: Vec<TlsPolicyConfig>,
    #[serde(default)]
    pub resolvers: Vec<ResolverConfig>,
    pub resources: Vec<ResourceConfig>,
    #[serde(default)]
    pub upstreams: Vec<UpstreamConfig>,
    pub virtual_hosts: Vec<VirtualHostConfig>,
    #[serde(default)]
    pub observability: ObservabilityConfig,
    #[serde(default)]
    pub deployment: DeploymentConfig,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompatibilityConfig {
    #[serde(default = "default_nginx_profile")]
    pub nginx_profile: String,
    #[serde(default = "default_unknown_directive_policy")]
    pub unknown_directive_policy: String,
}

impl Default for CompatibilityConfig {
    fn default() -> Self {
        Self {
            nginx_profile: default_nginx_profile(),
            unknown_directive_policy: default_unknown_directive_policy(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WebServerLimits {
    #[serde(default = "default_max_request_body_bytes")]
    pub max_request_body_bytes: u64,
    #[serde(default = "default_request_timeout_ms")]
    pub request_timeout_ms: u64,
    #[serde(default = "default_drain_timeout_ms")]
    pub drain_timeout_ms: u64,
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
}

impl Default for WebServerLimits {
    fn default() -> Self {
        Self {
            max_request_body_bytes: default_max_request_body_bytes(),
            request_timeout_ms: default_request_timeout_ms(),
            drain_timeout_ms: default_drain_timeout_ms(),
            max_connections: default_max_connections(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListenerConfig {
    pub id: String,
    pub bind: String,
    pub port: u16,
    pub protocols: Vec<ListenerProtocol>,
    pub tls_policy_ref: Option<String>,
    pub default_virtual_host_ref: Option<String>,
    pub max_connections: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListenerProtocol {
    Http1,
    Http2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CertificateConfig {
    pub id: String,
    pub server_names: Vec<String>,
    pub source: CertificateSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum CertificateSource {
    ProtectedFile {
        certificate_file: String,
        private_key_file: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TlsPolicyConfig {
    pub id: String,
    pub certificate_ref: String,
    #[serde(default = "default_tls_minimum")]
    pub minimum_version: TlsVersion,
    #[serde(default = "default_tls_maximum")]
    pub maximum_version: TlsVersion,
    #[serde(default = "default_alpn")]
    pub alpn: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TlsVersion {
    #[serde(rename = "tls1.2")]
    Tls12,
    #[serde(rename = "tls1.3")]
    Tls13,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolverConfig {
    pub id: String,
    #[serde(default)]
    pub servers: Vec<String>,
    #[serde(default = "default_resolver_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_maximum_answers")]
    pub maximum_answers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ResourceConfig {
    Static {
        id: String,
        root: String,
        #[serde(default = "default_index_files")]
        index_files: Vec<String>,
        spa_fallback: Option<String>,
        #[serde(default)]
        follow_symlinks: bool,
    },
    Proxy {
        id: String,
        upstream_ref: String,
        #[serde(default)]
        strip_prefix: bool,
    },
    Redirect {
        id: String,
        status: u16,
        location: String,
    },
    Respond {
        id: String,
        status: u16,
        #[serde(default = "default_content_type")]
        content_type: String,
        #[serde(default)]
        body: String,
    },
}

impl ResourceConfig {
    pub fn id(&self) -> &str {
        match self {
            Self::Static { id, .. }
            | Self::Proxy { id, .. }
            | Self::Redirect { id, .. }
            | Self::Respond { id, .. } => id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpstreamConfig {
    pub id: String,
    pub targets: Vec<UpstreamTargetConfig>,
    #[serde(default = "default_connect_timeout_ms")]
    pub connect_timeout_ms: u64,
    #[serde(default = "default_request_timeout_ms")]
    pub request_timeout_ms: u64,
    #[serde(default = "default_max_idle_connections")]
    pub max_idle_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpstreamTargetConfig {
    pub url: String,
    #[serde(default = "default_weight")]
    pub weight: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VirtualHostConfig {
    pub id: String,
    pub listener_refs: Vec<String>,
    pub server_names: Vec<String>,
    pub routes: Vec<RouteConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RouteConfig {
    pub id: String,
    #[serde(rename = "match")]
    pub route_match: RouteMatchConfig,
    pub resource_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RouteMatchConfig {
    pub path_type: RoutePathType,
    pub path: String,
    pub methods: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoutePathType {
    Exact,
    Prefix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObservabilityConfig {
    #[serde(default = "default_access_log")]
    pub access_log: bool,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            access_log: default_access_log(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeploymentConfig {
    pub drain_timeout_ms: Option<u64>,
}
