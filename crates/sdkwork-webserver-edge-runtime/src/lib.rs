//! Edge node runtime: nginx site paths, certificate bundle materialization, reload.

mod config;
mod error;
mod nginx;
mod paths;

pub use config::EdgeRuntimeConfig;
pub use error::{EdgeRuntimeError, EdgeRuntimeResult};
pub use nginx::{deploy_nginx_config, reload_nginx, validate_nginx_config};
pub use paths::{cert_bundle_paths, nginx_site_path};

use sdkwork_webserver_acme_service::IssuedCertificateMaterial;

pub struct EdgeRuntime {
    config: EdgeRuntimeConfig,
}

impl EdgeRuntime {
    pub fn from_env() -> Result<Self, EdgeRuntimeError> {
        Ok(Self {
            config: EdgeRuntimeConfig::from_env()?,
        })
    }

    pub fn config(&self) -> &EdgeRuntimeConfig {
        &self.config
    }

    pub fn write_certificate_bundle(
        &self,
        material: &IssuedCertificateMaterial,
    ) -> Result<(), EdgeRuntimeError> {
        paths::write_certificate_bundle(&self.config.cert_live_root, material)
    }

    pub fn deploy_site_config(
        &self,
        domain: &str,
        config_content: &str,
    ) -> Result<(), EdgeRuntimeError> {
        deploy_nginx_config(&self.config, domain, config_content)
    }

    pub fn validate_config_content(&self, config_content: &str) -> Result<(), EdgeRuntimeError> {
        validate_nginx_config(&self.config, config_content)
    }

    pub fn reload(&self) -> Result<(), EdgeRuntimeError> {
        reload_nginx(&self.config)
    }
}
