//! Framework-independent Web Server configuration and runtime helpers.

pub mod config;
pub mod runtime_env;

pub use config::{
    inspect_webserver_config_revision, load_and_compile_webserver_config,
    load_and_compile_webserver_config_revision, normalize_authority_host, normalize_server_name,
    normalize_uri_path, server_name_covers, upstream_ip_is_allowed, CertificateConfig,
    CertificateSource, CompiledWebServerApp, CompiledWebServerRevision, ConfigDiagnostic,
    ListenerConfig, ListenerProtocol, ProxyProtocolConfig, ProxyProtocolCrc32cPolicy,
    ProxyProtocolVersion, ReloadConfig, ReloadMode, ResolverConfig, ResourceConfig,
    ResourcePressureConfig, ResourceSampleFailurePolicy, RouteConfig, RouteMatchConfig,
    RoutePathType, SelectedRoute, TlsPolicyConfig, TlsVersion, TrustedProxyConfig,
    TrustedProxyHeader, UpstreamActiveHealthConfig, UpstreamActiveHealthMethod,
    UpstreamAddressPolicyConfig, UpstreamConfig, UpstreamLoadBalancingStrategy,
    UpstreamPassiveHealthConfig, UpstreamRetryCondition, UpstreamRetryConfig, UpstreamTlsConfig,
    UpstreamTlsTrustMode, UriPathNormalizationError, VirtualHostConfig, WebServerAppConfig,
    WebServerConfigError, WebServerConfigFileRevision, WebServerLimits, MAX_CONFIG_BYTES,
};
pub use runtime_env::{
    web_dev_auth_bypass_enabled, web_environment_name, web_is_production_like_environment,
    web_use_dev_inline_auth_resolver,
};
