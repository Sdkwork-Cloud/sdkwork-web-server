//! Framework-independent Web Server configuration and runtime helpers.

pub mod config;
pub mod runtime_env;

pub use config::{
    load_and_compile_webserver_config, normalize_authority_host, CertificateConfig,
    CertificateSource, CompiledWebServerApp, ConfigDiagnostic, ListenerConfig, ListenerProtocol,
    ResourceConfig, RouteConfig, RouteMatchConfig, RoutePathType, SelectedRoute, TlsPolicyConfig,
    UpstreamConfig, VirtualHostConfig, WebServerAppConfig, WebServerConfigError, WebServerLimits,
    MAX_CONFIG_BYTES,
};
pub use runtime_env::{
    web_dev_auth_bypass_enabled, web_environment_name, web_is_production_like_environment,
    web_use_dev_inline_auth_resolver,
};
