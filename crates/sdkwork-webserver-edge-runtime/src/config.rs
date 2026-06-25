use std::path::PathBuf;

use crate::EdgeRuntimeResult;

#[derive(Clone, Debug)]
pub struct EdgeRuntimeConfig {
    pub nginx_enabled: bool,
    pub nginx_binary: String,
    pub nginx_sites_root: PathBuf,
    pub cert_live_root: PathBuf,
    pub site_family: String,
}

impl EdgeRuntimeConfig {
    pub fn from_env() -> EdgeRuntimeResult<Self> {
        let nginx_enabled = std::env::var("SDKWORK_WEB_NGINX_ENABLED")
            .map(|value| value != "false" && value != "0")
            .unwrap_or(true);

        let nginx_binary =
            std::env::var("SDKWORK_WEB_NGINX_BINARY").unwrap_or_else(|_| "nginx".to_string());

        let nginx_sites_root = PathBuf::from(
            std::env::var("SDKWORK_WEB_NGINX_SITES_ROOT")
                .unwrap_or_else(|_| "/etc/nginx/sites-enabled/sdkwork".to_string()),
        );

        let cert_live_root = PathBuf::from(
            std::env::var("SDKWORK_WEB_CERT_LIVE_ROOT")
                .unwrap_or_else(|_| "/opt/certs/letsencrypt/live".to_string()),
        );

        let site_family = std::env::var("SDKWORK_WEB_NGINX_SITE_FAMILY")
            .unwrap_or_else(|_| "sdkwork".to_string());

        Ok(Self {
            nginx_enabled,
            nginx_binary,
            nginx_sites_root,
            cert_live_root,
            site_family,
        })
    }
}
