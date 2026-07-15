mod compiled;
mod error;
mod loader;
mod model;
mod validate;

pub use compiled::{normalize_authority_host, CompiledWebServerApp, SelectedRoute};
pub use error::{ConfigDiagnostic, WebServerConfigError};
pub use loader::{load_and_compile_webserver_config, MAX_CONFIG_BYTES};
pub use model::{
    CertificateConfig, CertificateSource, CompatibilityConfig, DeploymentConfig, ListenerConfig,
    ListenerProtocol, ObservabilityConfig, ResolverConfig, ResourceConfig, RouteConfig,
    RouteMatchConfig, RoutePathType, TlsPolicyConfig, TlsVersion, UpstreamConfig,
    UpstreamTargetConfig, VirtualHostConfig, WebServerAppConfig, WebServerLimits,
};

pub(crate) use validate::validate_webserver_config;
