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
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct EdgeRuntime {
    config: EdgeRuntimeConfig,
    certificate_activation_admission: Arc<Semaphore>,
}

impl EdgeRuntime {
    pub fn new(config: EdgeRuntimeConfig) -> Self {
        Self {
            config,
            certificate_activation_admission: Arc::new(Semaphore::new(1)),
        }
    }

    pub fn from_env() -> Result<Self, EdgeRuntimeError> {
        Ok(Self::new(EdgeRuntimeConfig::from_env()?))
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

    pub async fn write_certificate_bundle_async(
        &self,
        material: &IssuedCertificateMaterial,
    ) -> Result<(), EdgeRuntimeError> {
        let permit = self
            .certificate_activation_admission
            .clone()
            .try_acquire_owned()
            .map_err(|_| {
                EdgeRuntimeError::Filesystem(
                    "certificate bundle activation capacity exhausted".to_string(),
                )
            })?;
        let cert_live_root = self.config.cert_live_root.clone();
        let material = material.clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            paths::write_certificate_bundle(&cert_live_root, &material)
        })
        .await
        .map_err(|error| {
            EdgeRuntimeError::Filesystem(format!("certificate bundle task failed: {error}"))
        })?
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use sdkwork_webserver_acme_service::IssuedCertificateMaterial;
    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn async_certificate_activation_has_no_waiter_queue() {
        let root = TempDir::new().expect("tempdir");
        let runtime = EdgeRuntime::new(EdgeRuntimeConfig {
            nginx_enabled: false,
            nginx_binary: "nginx".to_string(),
            nginx_main_config: PathBuf::from("nginx.conf"),
            nginx_sites_root: root.path().join("sites"),
            cert_live_root: root.path().join("certs"),
            site_family: "sdkwork".to_string(),
            nginx_command_timeout_ms: 10_000,
        });
        let permit = runtime
            .certificate_activation_admission
            .clone()
            .try_acquire_owned()
            .expect("permit");
        let invalid = IssuedCertificateMaterial {
            cert_name: "cert-id".to_string(),
            cert_type: 3,
            issuer: String::new(),
            subject: String::new(),
            san_list: String::new(),
            fingerprint: String::new(),
            cert_pem: "invalid".to_string(),
            private_key_pem: "invalid".to_string(),
            chain_pem: None,
            not_before: String::new(),
            not_after: String::new(),
            cert_path: String::new(),
            key_path: String::new(),
            chain_path: None,
        };
        let error = runtime
            .write_certificate_bundle_async(&invalid)
            .await
            .expect_err("capacity must fail closed");
        assert!(error.to_string().contains("capacity exhausted"));
        drop(permit);
        let error = runtime
            .write_certificate_bundle_async(&invalid)
            .await
            .expect_err("material validation must run after capacity recovers");
        assert!(error.to_string().contains("certificate PEM"));
    }
}
