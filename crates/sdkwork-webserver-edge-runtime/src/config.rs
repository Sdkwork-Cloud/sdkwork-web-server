use std::path::PathBuf;

use crate::EdgeRuntimeResult;

#[derive(Clone, Debug)]
pub struct EdgeRuntimeConfig {
    pub nginx_enabled: bool,
    pub nginx_binary: String,
    pub nginx_main_config: PathBuf,
    pub nginx_sites_root: PathBuf,
    pub cert_live_root: PathBuf,
    pub site_family: String,
    pub nginx_command_timeout_ms: u64,
}

impl EdgeRuntimeConfig {
    pub fn from_env() -> EdgeRuntimeResult<Self> {
        let nginx_enabled = match std::env::var("SDKWORK_WEB_NGINX_ENABLED") {
            Ok(value) => parse_enabled(&value)?,
            Err(std::env::VarError::NotPresent) => true,
            Err(std::env::VarError::NotUnicode(_)) => {
                return Err(crate::EdgeRuntimeError::Config(
                    "SDKWORK_WEB_NGINX_ENABLED must be valid Unicode".to_string(),
                ));
            }
        };

        let nginx_binary =
            std::env::var("SDKWORK_WEB_NGINX_BINARY").unwrap_or_else(|_| "nginx".to_string());

        let nginx_main_config = PathBuf::from(
            std::env::var("SDKWORK_WEB_NGINX_MAIN_CONF")
                .unwrap_or_else(|_| "/etc/nginx/nginx.conf".to_string()),
        );

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

        let nginx_command_timeout_ms = std::env::var("SDKWORK_WEB_NGINX_COMMAND_TIMEOUT_MS")
            .ok()
            .map(|value| {
                value.parse::<u64>().map_err(|error| {
                    crate::EdgeRuntimeError::Config(format!(
                        "invalid SDKWORK_WEB_NGINX_COMMAND_TIMEOUT_MS: {error}"
                    ))
                })
            })
            .transpose()?
            .unwrap_or(10_000);
        if !(100..=60_000).contains(&nginx_command_timeout_ms) {
            return Err(crate::EdgeRuntimeError::Config(
                "SDKWORK_WEB_NGINX_COMMAND_TIMEOUT_MS must be between 100 and 60000".to_string(),
            ));
        }

        Ok(Self {
            nginx_enabled,
            nginx_binary,
            nginx_main_config,
            nginx_sites_root,
            cert_live_root,
            site_family,
            nginx_command_timeout_ms,
        })
    }
}

fn parse_enabled(value: &str) -> EdgeRuntimeResult<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(crate::EdgeRuntimeError::Config(
            "SDKWORK_WEB_NGINX_ENABLED must be true, false, 1, or 0".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_enabled;

    #[test]
    fn nginx_enabled_tokens_are_strict() {
        assert!(parse_enabled("true").unwrap());
        assert!(parse_enabled("1").unwrap());
        assert!(!parse_enabled("false").unwrap());
        assert!(!parse_enabled("0").unwrap());
        assert!(parse_enabled("yes").is_err());
        assert!(parse_enabled("").is_err());
    }
}
