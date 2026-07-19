mod compiled;
mod error;
mod loader;
mod model;
mod network;
mod uri;
mod validate;

pub use compiled::{normalize_authority_host, CompiledWebServerApp, SelectedRoute};
pub use error::{ConfigDiagnostic, WebServerConfigError};
pub use loader::{
    inspect_webserver_config_revision, load_and_compile_webserver_config,
    load_and_compile_webserver_config_revision, CompiledWebServerRevision,
    WebServerConfigFileRevision, MAX_CONFIG_BYTES,
};
pub use model::{
    CertificateConfig, CertificateSource, CompatibilityConfig, DeploymentConfig, ListenerConfig,
    ListenerProtocol, ObservabilityConfig, ProxyProtocolConfig, ProxyProtocolCrc32cPolicy,
    ProxyProtocolVersion, ReloadConfig, ReloadMode, ResolverConfig, ResourceConfig,
    ResourcePressureConfig, ResourceSampleFailurePolicy, RouteConfig, RouteMatchConfig,
    RoutePathType, TlsPolicyConfig, TlsVersion, TrustedProxyConfig, TrustedProxyHeader,
    UpstreamActiveHealthConfig, UpstreamActiveHealthMethod, UpstreamAddressPolicyConfig,
    UpstreamConfig, UpstreamLoadBalancingStrategy, UpstreamPassiveHealthConfig,
    UpstreamRetryCondition, UpstreamRetryConfig, UpstreamTargetConfig, UpstreamTlsConfig,
    UpstreamTlsTrustMode, VirtualHostConfig, WebServerAppConfig, WebServerLimits,
};
pub use network::{is_supported_upstream_allowed_cidr, upstream_ip_is_allowed};
pub use uri::{normalize_uri_path, UriPathNormalizationError};
pub use validate::{normalize_server_name, server_name_covers};

pub(crate) use validate::validate_webserver_config;
