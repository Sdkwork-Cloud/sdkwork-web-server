use std::collections::BTreeMap;

use ipnet::IpNet;
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

fn default_request_body_start_timeout_ms() -> u64 {
    30_000
}

fn default_request_body_idle_timeout_ms() -> u64 {
    30_000
}

fn default_response_body_idle_timeout_ms() -> u64 {
    30_000
}

fn default_connection_write_timeout_ms() -> u64 {
    30_000
}

fn default_http1_keep_alive_idle_timeout_ms() -> u64 {
    75_000
}

fn default_http1_max_pipeline_depth() -> usize {
    16
}

fn default_drain_timeout_ms() -> u64 {
    30_000
}

fn default_max_connections() -> usize {
    10_000
}

fn default_max_concurrent_requests() -> usize {
    4_096
}

fn default_max_request_header_bytes() -> usize {
    64 * 1024
}

fn default_max_request_line_bytes() -> usize {
    8 * 1024
}

fn default_max_request_method_bytes() -> usize {
    32
}

fn default_max_request_target_bytes() -> usize {
    8 * 1024
}

fn default_max_uri_path_bytes() -> usize {
    8 * 1024
}

fn default_max_decoded_path_bytes() -> usize {
    8 * 1024
}

fn default_max_path_segments() -> usize {
    256
}

fn default_max_query_string_bytes() -> usize {
    4 * 1024
}

fn default_max_query_parameters() -> usize {
    256
}

fn default_max_query_component_bytes() -> usize {
    1024
}

fn default_max_header_name_bytes() -> usize {
    256
}

fn default_max_header_value_bytes() -> usize {
    8 * 1024
}

fn default_max_request_headers() -> usize {
    100
}

fn default_request_header_timeout_ms() -> u64 {
    10_000
}

fn default_max_chunk_line_bytes() -> usize {
    1_024
}

fn default_max_trailer_bytes() -> usize {
    8 * 1024
}

fn default_max_trailers() -> usize {
    32
}

fn default_http2_max_concurrent_streams() -> u32 {
    100
}

fn default_max_connection_age_ms() -> u64 {
    3_600_000
}

fn default_http2_keep_alive_interval_ms() -> u64 {
    60_000
}

fn default_http2_keep_alive_timeout_ms() -> u64 {
    20_000
}

fn default_http2_max_pending_accept_reset_streams() -> usize {
    20
}

fn default_http2_max_local_error_reset_streams() -> usize {
    128
}

fn default_http2_max_send_buffer_bytes() -> usize {
    64 * 1024
}

fn default_http2_max_header_list_bytes() -> u32 {
    64 * 1024
}

fn default_http2_max_frame_bytes() -> u32 {
    16 * 1024
}

fn default_http2_abuse_window_ms() -> u64 {
    1_000
}

fn default_http2_max_frames_per_window() -> usize {
    10_000
}

fn default_http2_max_new_streams_per_window() -> usize {
    1_000
}

fn default_http2_max_reset_frames_per_window() -> usize {
    100
}

fn default_http2_max_continuation_frames() -> usize {
    16
}

fn default_http2_max_encoded_header_block_bytes() -> usize {
    64 * 1024
}

fn default_trusted_proxy_max_hops() -> usize {
    16
}

fn default_trusted_proxy_max_header_bytes() -> usize {
    4 * 1024
}

fn default_proxy_protocol_versions() -> Vec<ProxyProtocolVersion> {
    vec![ProxyProtocolVersion::V1, ProxyProtocolVersion::V2]
}

fn default_proxy_protocol_timeout_ms() -> u64 {
    3_000
}

fn default_proxy_protocol_max_header_bytes() -> usize {
    536
}

fn default_proxy_protocol_crc32c_policy() -> ProxyProtocolCrc32cPolicy {
    ProxyProtocolCrc32cPolicy::Ignore
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

fn default_upstream_max_connections() -> usize {
    256
}

fn default_upstream_max_response_header_bytes() -> usize {
    64 * 1024
}

fn default_upstream_max_response_headers() -> usize {
    100
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

fn default_max_concurrent_queries() -> usize {
    64
}

fn default_idle_connection_timeout_ms() -> u64 {
    30_000
}

fn default_upstream_max_in_flight_requests() -> usize {
    1_024
}

fn default_upstream_retry_max_attempts() -> u8 {
    2
}

fn default_upstream_retry_timeout_ms() -> u64 {
    30_000
}

fn default_upstream_retry_on() -> Vec<UpstreamRetryCondition> {
    vec![
        UpstreamRetryCondition::TransportFailure,
        UpstreamRetryCondition::Timeout,
    ]
}

fn default_max_concurrent_health_checks() -> usize {
    64
}

fn default_passive_failure_threshold() -> u32 {
    3
}

fn default_passive_ejection_time_ms() -> u64 {
    30_000
}

fn default_passive_failure_statuses() -> Vec<u16> {
    vec![502, 503, 504]
}

fn default_active_health_uri() -> String {
    "/".to_owned()
}

fn default_active_health_interval_ms() -> u64 {
    10_000
}

fn default_active_health_timeout_ms() -> u64 {
    2_000
}

fn default_active_unhealthy_threshold() -> u32 {
    3
}

fn default_active_healthy_threshold() -> u32 {
    2
}

fn default_active_success_status_min() -> u16 {
    200
}

fn default_active_success_status_max() -> u16 {
    399
}

fn default_active_max_response_body_bytes() -> u64 {
    65_536
}

fn default_access_log() -> bool {
    true
}

fn default_reload_poll_interval_ms() -> u64 {
    1_000
}

fn default_resource_sample_interval_ms() -> u64 {
    250
}

fn default_maximum_process_memory_bytes() -> u64 {
    1_073_741_824
}

fn default_memory_reserve_bytes() -> u64 {
    67_108_864
}

fn default_memory_admission_percent() -> u8 {
    90
}

fn default_memory_recovery_percent() -> u8 {
    80
}

fn default_maximum_open_handles() -> u64 {
    16_384
}

fn default_open_handle_reserve() -> u64 {
    128
}

fn default_open_handle_admission_percent() -> u8 {
    90
}

fn default_open_handle_recovery_percent() -> u8 {
    80
}

fn default_event_loop_lag_admission_ms() -> u64 {
    250
}

fn default_event_loop_lag_recovery_ms() -> u64 {
    50
}

fn default_consecutive_pressure_samples() -> u32 {
    2
}

fn default_consecutive_recovery_samples() -> u32 {
    4
}

fn default_operations_reserve_requests() -> usize {
    16
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WebServerLimits {
    #[serde(default = "default_max_request_body_bytes")]
    pub max_request_body_bytes: u64,
    #[serde(default = "default_request_timeout_ms")]
    pub request_timeout_ms: u64,
    #[serde(default = "default_request_body_start_timeout_ms")]
    pub request_body_start_timeout_ms: u64,
    #[serde(default = "default_request_body_idle_timeout_ms")]
    pub request_body_idle_timeout_ms: u64,
    #[serde(default = "default_response_body_idle_timeout_ms")]
    pub response_body_idle_timeout_ms: u64,
    #[serde(default = "default_connection_write_timeout_ms")]
    pub connection_write_timeout_ms: u64,
    #[serde(default = "default_http1_keep_alive_idle_timeout_ms")]
    pub http1_keep_alive_idle_timeout_ms: u64,
    #[serde(default = "default_http1_max_pipeline_depth")]
    pub http1_max_pipeline_depth: usize,
    #[serde(default = "default_drain_timeout_ms")]
    pub drain_timeout_ms: u64,
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    #[serde(default = "default_max_concurrent_requests")]
    pub max_concurrent_requests: usize,
    #[serde(default = "default_max_concurrent_health_checks")]
    pub max_concurrent_health_checks: usize,
    #[serde(default = "default_max_request_header_bytes")]
    pub max_request_header_bytes: usize,
    #[serde(default = "default_max_request_line_bytes")]
    pub max_request_line_bytes: usize,
    #[serde(default = "default_max_request_method_bytes")]
    pub max_request_method_bytes: usize,
    #[serde(default = "default_max_request_target_bytes")]
    pub max_request_target_bytes: usize,
    #[serde(default = "default_max_uri_path_bytes")]
    pub max_uri_path_bytes: usize,
    #[serde(default = "default_max_decoded_path_bytes")]
    pub max_decoded_path_bytes: usize,
    #[serde(default = "default_max_path_segments")]
    pub max_path_segments: usize,
    #[serde(default = "default_max_query_string_bytes")]
    pub max_query_string_bytes: usize,
    #[serde(default = "default_max_query_parameters")]
    pub max_query_parameters: usize,
    #[serde(default = "default_max_query_component_bytes")]
    pub max_query_component_bytes: usize,
    #[serde(default = "default_max_header_name_bytes")]
    pub max_header_name_bytes: usize,
    #[serde(default = "default_max_header_value_bytes")]
    pub max_header_value_bytes: usize,
    #[serde(default = "default_max_request_headers")]
    pub max_request_headers: usize,
    #[serde(default = "default_request_header_timeout_ms")]
    pub request_header_timeout_ms: u64,
    #[serde(default = "default_max_chunk_line_bytes")]
    pub max_chunk_line_bytes: usize,
    #[serde(default = "default_max_trailer_bytes")]
    pub max_trailer_bytes: usize,
    #[serde(default = "default_max_trailers")]
    pub max_trailers: usize,
    #[serde(default = "default_http2_max_concurrent_streams")]
    pub http2_max_concurrent_streams: u32,
    #[serde(default = "default_max_connection_age_ms")]
    pub max_connection_age_ms: u64,
    #[serde(default = "default_http2_keep_alive_interval_ms")]
    pub http2_keep_alive_interval_ms: u64,
    #[serde(default = "default_http2_keep_alive_timeout_ms")]
    pub http2_keep_alive_timeout_ms: u64,
    #[serde(default = "default_http2_max_pending_accept_reset_streams")]
    pub http2_max_pending_accept_reset_streams: usize,
    #[serde(default = "default_http2_max_local_error_reset_streams")]
    pub http2_max_local_error_reset_streams: usize,
    #[serde(default = "default_http2_max_send_buffer_bytes")]
    pub http2_max_send_buffer_bytes: usize,
    #[serde(default = "default_http2_max_header_list_bytes")]
    pub http2_max_header_list_bytes: u32,
    #[serde(default = "default_http2_max_frame_bytes")]
    pub http2_max_frame_bytes: u32,
    #[serde(default = "default_http2_abuse_window_ms")]
    pub http2_abuse_window_ms: u64,
    #[serde(default = "default_http2_max_frames_per_window")]
    pub http2_max_frames_per_window: usize,
    #[serde(default = "default_http2_max_new_streams_per_window")]
    pub http2_max_new_streams_per_window: usize,
    #[serde(default = "default_http2_max_reset_frames_per_window")]
    pub http2_max_reset_frames_per_window: usize,
    #[serde(default = "default_http2_max_continuation_frames")]
    pub http2_max_continuation_frames: usize,
    #[serde(default = "default_http2_max_encoded_header_block_bytes")]
    pub http2_max_encoded_header_block_bytes: usize,
}

impl Default for WebServerLimits {
    fn default() -> Self {
        Self {
            max_request_body_bytes: default_max_request_body_bytes(),
            request_timeout_ms: default_request_timeout_ms(),
            request_body_start_timeout_ms: default_request_body_start_timeout_ms(),
            request_body_idle_timeout_ms: default_request_body_idle_timeout_ms(),
            response_body_idle_timeout_ms: default_response_body_idle_timeout_ms(),
            connection_write_timeout_ms: default_connection_write_timeout_ms(),
            http1_keep_alive_idle_timeout_ms: default_http1_keep_alive_idle_timeout_ms(),
            http1_max_pipeline_depth: default_http1_max_pipeline_depth(),
            drain_timeout_ms: default_drain_timeout_ms(),
            max_connections: default_max_connections(),
            max_concurrent_requests: default_max_concurrent_requests(),
            max_concurrent_health_checks: default_max_concurrent_health_checks(),
            max_request_header_bytes: default_max_request_header_bytes(),
            max_request_line_bytes: default_max_request_line_bytes(),
            max_request_method_bytes: default_max_request_method_bytes(),
            max_request_target_bytes: default_max_request_target_bytes(),
            max_uri_path_bytes: default_max_uri_path_bytes(),
            max_decoded_path_bytes: default_max_decoded_path_bytes(),
            max_path_segments: default_max_path_segments(),
            max_query_string_bytes: default_max_query_string_bytes(),
            max_query_parameters: default_max_query_parameters(),
            max_query_component_bytes: default_max_query_component_bytes(),
            max_header_name_bytes: default_max_header_name_bytes(),
            max_header_value_bytes: default_max_header_value_bytes(),
            max_request_headers: default_max_request_headers(),
            request_header_timeout_ms: default_request_header_timeout_ms(),
            max_chunk_line_bytes: default_max_chunk_line_bytes(),
            max_trailer_bytes: default_max_trailer_bytes(),
            max_trailers: default_max_trailers(),
            http2_max_concurrent_streams: default_http2_max_concurrent_streams(),
            max_connection_age_ms: default_max_connection_age_ms(),
            http2_keep_alive_interval_ms: default_http2_keep_alive_interval_ms(),
            http2_keep_alive_timeout_ms: default_http2_keep_alive_timeout_ms(),
            http2_max_pending_accept_reset_streams: default_http2_max_pending_accept_reset_streams(
            ),
            http2_max_local_error_reset_streams: default_http2_max_local_error_reset_streams(),
            http2_max_send_buffer_bytes: default_http2_max_send_buffer_bytes(),
            http2_max_header_list_bytes: default_http2_max_header_list_bytes(),
            http2_max_frame_bytes: default_http2_max_frame_bytes(),
            http2_abuse_window_ms: default_http2_abuse_window_ms(),
            http2_max_frames_per_window: default_http2_max_frames_per_window(),
            http2_max_new_streams_per_window: default_http2_max_new_streams_per_window(),
            http2_max_reset_frames_per_window: default_http2_max_reset_frames_per_window(),
            http2_max_continuation_frames: default_http2_max_continuation_frames(),
            http2_max_encoded_header_block_bytes: default_http2_max_encoded_header_block_bytes(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListenerConfig {
    pub id: String,
    pub bind: String,
    pub port: u16,
    pub protocols: Vec<ListenerProtocol>,
    pub tls_policy_ref: Option<String>,
    pub default_virtual_host_ref: Option<String>,
    pub max_connections: Option<usize>,
    pub trusted_proxy: Option<TrustedProxyConfig>,
    pub proxy_protocol: Option<ProxyProtocolConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListenerProtocol {
    Http1,
    Http2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrustedProxyConfig {
    pub trusted_cidrs: Vec<IpNet>,
    #[serde(default)]
    pub header: TrustedProxyHeader,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default = "default_trusted_proxy_max_hops")]
    pub max_hops: usize,
    #[serde(default = "default_trusted_proxy_max_header_bytes")]
    pub max_header_bytes: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrustedProxyHeader {
    #[default]
    XForwardedFor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProxyProtocolConfig {
    pub trusted_source_cidrs: Vec<IpNet>,
    #[serde(default = "default_proxy_protocol_versions")]
    pub versions: Vec<ProxyProtocolVersion>,
    #[serde(default = "default_proxy_protocol_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_proxy_protocol_max_header_bytes")]
    pub max_header_bytes: usize,
    #[serde(default = "default_proxy_protocol_crc32c_policy")]
    pub crc32c_policy: ProxyProtocolCrc32cPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyProtocolVersion {
    V1,
    V2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProxyProtocolCrc32cPolicy {
    Ignore,
    ValidateIfPresent,
    Required,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CertificateConfig {
    pub id: String,
    pub server_names: Vec<String>,
    pub source: CertificateSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TlsPolicyConfig {
    pub id: String,
    pub certificate_ref: Option<String>,
    #[serde(default)]
    pub certificate_refs: Vec<String>,
    #[serde(default = "default_tls_minimum")]
    pub minimum_version: TlsVersion,
    #[serde(default = "default_tls_maximum")]
    pub maximum_version: TlsVersion,
    #[serde(default = "default_alpn")]
    pub alpn: Vec<String>,
}

impl TlsPolicyConfig {
    pub fn certificate_refs(&self) -> impl Iterator<Item = &str> {
        self.certificate_ref
            .iter()
            .map(String::as_str)
            .chain(self.certificate_refs.iter().map(String::as_str))
    }
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
    #[serde(default = "default_max_concurrent_queries")]
    pub max_concurrent_queries: usize,
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
    #[serde(default)]
    pub load_balancing: UpstreamLoadBalancingStrategy,
    pub resolver_ref: Option<String>,
    #[serde(default)]
    pub address_policy: UpstreamAddressPolicyConfig,
    pub tls: Option<UpstreamTlsConfig>,
    #[serde(default = "default_connect_timeout_ms")]
    pub connect_timeout_ms: u64,
    #[serde(default = "default_request_timeout_ms")]
    pub request_timeout_ms: u64,
    #[serde(default = "default_max_idle_connections")]
    pub max_idle_connections: usize,
    #[serde(default = "default_upstream_max_connections")]
    pub max_connections: usize,
    #[serde(default = "default_upstream_max_response_header_bytes")]
    pub max_response_header_bytes: usize,
    #[serde(default = "default_upstream_max_response_headers")]
    pub max_response_headers: usize,
    #[serde(default = "default_idle_connection_timeout_ms")]
    pub idle_connection_timeout_ms: u64,
    #[serde(default = "default_upstream_max_in_flight_requests")]
    pub max_in_flight_requests: usize,
    pub retry: Option<UpstreamRetryConfig>,
    #[serde(default)]
    pub passive_health: UpstreamPassiveHealthConfig,
    pub active_health: Option<UpstreamActiveHealthConfig>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpstreamLoadBalancingStrategy {
    #[default]
    RoundRobin,
    LeastConnections,
    RandomTwoLeastConnections,
    IpHash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpstreamRetryConfig {
    #[serde(default = "default_upstream_retry_max_attempts")]
    pub max_attempts: u8,
    #[serde(default = "default_upstream_retry_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_upstream_retry_on")]
    pub retry_on: Vec<UpstreamRetryCondition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpstreamRetryCondition {
    #[serde(rename = "error")]
    TransportFailure,
    Timeout,
    #[serde(rename = "http_502")]
    Http502,
    #[serde(rename = "http_503")]
    Http503,
    #[serde(rename = "http_504")]
    Http504,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpstreamAddressPolicyConfig {
    #[serde(default)]
    pub allowed_cidrs: Vec<IpNet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpstreamTlsConfig {
    #[serde(default)]
    pub trust_mode: UpstreamTlsTrustMode,
    #[serde(default)]
    pub ca_certificate_files: Vec<String>,
    pub client_certificate_file: Option<String>,
    pub client_private_key_file: Option<String>,
    #[serde(default = "default_tls_minimum")]
    pub minimum_version: TlsVersion,
    #[serde(default = "default_tls_maximum")]
    pub maximum_version: TlsVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpstreamPassiveHealthConfig {
    #[serde(default = "default_passive_failure_threshold")]
    pub failure_threshold: u32,
    #[serde(default = "default_passive_ejection_time_ms")]
    pub ejection_time_ms: u64,
    #[serde(default = "default_passive_failure_statuses")]
    pub failure_statuses: Vec<u16>,
}

impl Default for UpstreamPassiveHealthConfig {
    fn default() -> Self {
        Self {
            failure_threshold: default_passive_failure_threshold(),
            ejection_time_ms: default_passive_ejection_time_ms(),
            failure_statuses: default_passive_failure_statuses(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpstreamActiveHealthConfig {
    #[serde(default)]
    pub method: UpstreamActiveHealthMethod,
    #[serde(default = "default_active_health_uri")]
    pub uri: String,
    #[serde(default = "default_active_health_interval_ms")]
    pub interval_ms: u64,
    #[serde(default = "default_active_health_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_active_unhealthy_threshold")]
    pub unhealthy_threshold: u32,
    #[serde(default = "default_active_healthy_threshold")]
    pub healthy_threshold: u32,
    #[serde(default = "default_active_success_status_min")]
    pub success_status_min: u16,
    #[serde(default = "default_active_success_status_max")]
    pub success_status_max: u16,
    #[serde(default = "default_active_max_response_body_bytes")]
    pub max_response_body_bytes: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum UpstreamActiveHealthMethod {
    #[default]
    Get,
    Head,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpstreamTlsTrustMode {
    #[default]
    System,
    Custom,
    SystemAndCustom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpstreamTargetConfig {
    pub url: String,
    #[serde(default = "default_weight")]
    pub weight: u16,
    #[serde(default)]
    pub backup: bool,
    pub slow_start_ms: Option<u64>,
    pub max_connections: Option<usize>,
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeploymentConfig {
    pub drain_timeout_ms: Option<u64>,
    #[serde(default)]
    pub reload: ReloadConfig,
    pub resource_pressure: Option<ResourcePressureConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourcePressureConfig {
    #[serde(default = "default_resource_sample_interval_ms")]
    pub sample_interval_ms: u64,
    #[serde(default = "default_maximum_process_memory_bytes")]
    pub maximum_process_memory_bytes: u64,
    #[serde(default = "default_memory_reserve_bytes")]
    pub memory_reserve_bytes: u64,
    #[serde(default = "default_memory_admission_percent")]
    pub memory_admission_percent: u8,
    #[serde(default = "default_memory_recovery_percent")]
    pub memory_recovery_percent: u8,
    #[serde(default = "default_maximum_open_handles")]
    pub maximum_open_handles: u64,
    #[serde(default = "default_open_handle_reserve")]
    pub open_handle_reserve: u64,
    #[serde(default = "default_open_handle_admission_percent")]
    pub open_handle_admission_percent: u8,
    #[serde(default = "default_open_handle_recovery_percent")]
    pub open_handle_recovery_percent: u8,
    #[serde(default = "default_event_loop_lag_admission_ms")]
    pub event_loop_lag_admission_ms: u64,
    #[serde(default = "default_event_loop_lag_recovery_ms")]
    pub event_loop_lag_recovery_ms: u64,
    #[serde(default = "default_consecutive_pressure_samples")]
    pub consecutive_pressure_samples: u32,
    #[serde(default = "default_consecutive_recovery_samples")]
    pub consecutive_recovery_samples: u32,
    #[serde(default = "default_operations_reserve_requests")]
    pub operations_reserve_requests: usize,
    #[serde(default)]
    pub sample_failure_policy: ResourceSampleFailurePolicy,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResourceSampleFailurePolicy {
    FailOpen,
    #[default]
    FailClosed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReloadConfig {
    #[serde(default)]
    pub mode: ReloadMode,
    #[serde(default = "default_reload_poll_interval_ms")]
    pub poll_interval_ms: u64,
}

impl Default for ReloadConfig {
    fn default() -> Self {
        Self {
            mode: ReloadMode::Disabled,
            poll_interval_ms: default_reload_poll_interval_ms(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReloadMode {
    #[default]
    Disabled,
    Watch,
}
