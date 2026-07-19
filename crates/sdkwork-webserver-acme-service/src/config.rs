use std::path::Path;

use sdkwork_utils_rust::derive_aes_256_key;
use url::Url;

use crate::{AcmeServiceError, AcmeServiceResult};

const CERT_ENCRYPTION_KEY_INFO: &[u8] = b"sdkwork-web-acme-cert-encryption";
const MAX_DIRECTORY_URL_BYTES: usize = 2_048;
const MAX_CONTACT_EMAIL_BYTES: usize = 254;
const MAX_WEBROOT_BYTES: usize = 4_096;
const MIN_PRODUCTION_SECRET_BYTES: usize = 32;

pub const DEFAULT_ACME_OPERATION_TIMEOUT_MS: u64 = 180_000;
pub const MIN_ACME_OPERATION_TIMEOUT_MS: u64 = 10_000;
pub const MAX_ACME_OPERATION_TIMEOUT_MS: u64 = 600_000;

/// Validated runtime ACME configuration.
#[derive(Clone, Debug)]
pub struct AcmeConfig {
    pub directory_url: String,
    pub contact_email: String,
    pub renew_before_days: u32,
    pub webroot: Option<String>,
    pub encryption_key: Vec<u8>,
    pub use_production: bool,
}

impl AcmeConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        directory_url: String,
        contact_email: String,
        renew_before_days: u32,
        webroot: Option<String>,
        encryption_key_material: &[u8],
        use_production: bool,
        production_like: bool,
    ) -> AcmeServiceResult<Self> {
        validate_directory_url(&directory_url)?;
        validate_contact_email(&contact_email)?;
        if !(1..=90).contains(&renew_before_days) {
            return Err(AcmeServiceError::config(
                "certificate renewal window must be between 1 and 90 days",
            ));
        }
        if let Some(path) = webroot.as_deref() {
            validate_webroot(path)?;
        }
        let secure_profile = production_like || use_production;
        if encryption_key_material.is_empty()
            || (secure_profile && encryption_key_material.len() < MIN_PRODUCTION_SECRET_BYTES)
        {
            return Err(AcmeServiceError::config(if secure_profile {
                "certificate encryption key must contain at least 32 bytes in production-like environments"
            } else {
                "certificate encryption key must not be empty"
            }));
        }

        let encryption_key = derive_aes_256_key(
            encryption_key_material,
            b"sdkwork-web-acme",
            CERT_ENCRYPTION_KEY_INFO,
        )
        .to_vec();

        Ok(Self {
            directory_url,
            contact_email,
            renew_before_days,
            webroot,
            encryption_key,
            use_production,
        })
    }

    pub fn validate(&self) -> AcmeServiceResult<()> {
        validate_directory_url(&self.directory_url)?;
        validate_contact_email(&self.contact_email)?;
        if !(1..=90).contains(&self.renew_before_days) {
            return Err(AcmeServiceError::config(
                "certificate renewal window must be between 1 and 90 days",
            ));
        }
        if let Some(path) = self.webroot.as_deref() {
            validate_webroot(path)?;
        }
        if self.encryption_key.len() != 32 {
            return Err(AcmeServiceError::config(
                "derived certificate encryption key must contain exactly 32 bytes",
            ));
        }
        Ok(())
    }

    /// Compatibility loader. Deployable runtimes should inject typed configuration.
    pub fn from_env() -> AcmeServiceResult<Self> {
        let environment = std::env::var("SDKWORK_WEB_ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .to_ascii_lowercase();
        let use_production = match std::env::var("SDKWORK_WEB_ACME_PROFILE") {
            Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
                "production" | "prod" => true,
                "staging" | "stage" | "test" => false,
                other => {
                    return Err(AcmeServiceError::config(format!(
                        "invalid SDKWORK_WEB_ACME_PROFILE {other}; expected production or staging"
                    )));
                }
            },
            Err(_) => matches!(environment.as_str(), "production" | "prod"),
        };
        let production_like = use_production
            || matches!(
                environment.as_str(),
                "production" | "prod" | "staging" | "stage" | "test"
            );

        let directory_url = std::env::var("SDKWORK_WEB_ACME_DIRECTORY_URL").unwrap_or_else(|_| {
            if use_production {
                "https://acme-v02.api.letsencrypt.org/directory".to_string()
            } else {
                "https://acme-staging-v02.api.letsencrypt.org/directory".to_string()
            }
        });
        let contact_email = match std::env::var("SDKWORK_WEB_ACME_CONTACT_EMAIL") {
            Ok(value) => value,
            Err(_) if !production_like => "admin@localhost".to_string(),
            Err(_) => {
                return Err(AcmeServiceError::config(
                    "SDKWORK_WEB_ACME_CONTACT_EMAIL is required in production-like environments",
                ));
            }
        };
        let renew_before_days = std::env::var("SDKWORK_WEB_CERT_RENEW_BEFORE_DAYS")
            .map(|value| {
                value.parse::<u32>().map_err(|error| {
                    AcmeServiceError::config(format!(
                        "invalid SDKWORK_WEB_CERT_RENEW_BEFORE_DAYS: {error}"
                    ))
                })
            })
            .unwrap_or(Ok(30))?;
        let webroot = std::env::var("SDKWORK_WEB_ACME_WEBROOT").ok();
        let raw_key = match std::env::var("SDKWORK_WEB_CERT_ENCRYPTION_KEY") {
            Ok(value) => value,
            Err(_) if !production_like => {
                tracing::warn!(
                    "SDKWORK_WEB_CERT_ENCRYPTION_KEY missing; using development-only derived key"
                );
                "sdkwork-web-development-cert-key".to_string()
            }
            Err(_) => {
                return Err(AcmeServiceError::config(
                    "SDKWORK_WEB_CERT_ENCRYPTION_KEY is required in production-like environments",
                ));
            }
        };

        Self::new(
            directory_url,
            contact_email,
            renew_before_days,
            webroot,
            raw_key.as_bytes(),
            use_production,
            production_like,
        )
    }
}

fn validate_directory_url(value: &str) -> AcmeServiceResult<()> {
    if value.is_empty() || value.len() > MAX_DIRECTORY_URL_BYTES {
        return Err(AcmeServiceError::config(
            "ACME directory URL must contain 1..2048 bytes",
        ));
    }
    let url = Url::parse(value).map_err(|error| {
        AcmeServiceError::config(format!("invalid ACME directory URL: {error}"))
    })?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(AcmeServiceError::config(
            "ACME directory URL must be an HTTPS URL without userinfo",
        ));
    }
    Ok(())
}

fn validate_contact_email(value: &str) -> AcmeServiceResult<()> {
    if value.is_empty()
        || value.len() > MAX_CONTACT_EMAIL_BYTES
        || !value.is_ascii()
        || value
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control())
    {
        return Err(AcmeServiceError::config(
            "ACME contact email must contain 1..254 safe ASCII bytes",
        ));
    }
    let Some((local, domain)) = value.rsplit_once('@') else {
        return Err(AcmeServiceError::config("ACME contact email is invalid"));
    };
    if local.is_empty() || domain.is_empty() || domain.starts_with('.') || domain.ends_with('.') {
        return Err(AcmeServiceError::config("ACME contact email is invalid"));
    }
    Ok(())
}

fn validate_webroot(value: &str) -> AcmeServiceResult<()> {
    if value.is_empty()
        || value.len() > MAX_WEBROOT_BYTES
        || value
            .bytes()
            .any(|byte| byte == 0 || byte.is_ascii_control())
        || Path::new(value).as_os_str().is_empty()
    {
        return Err(AcmeServiceError::config(
            "ACME webroot must contain 1..4096 safe path bytes",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> AcmeServiceResult<AcmeConfig> {
        AcmeConfig::new(
            "https://acme-staging-v02.api.letsencrypt.org/directory".to_string(),
            "admin@example.com".to_string(),
            30,
            Some("/var/www/acme".to_string()),
            b"0123456789abcdef0123456789abcdef",
            false,
            true,
        )
    }

    #[test]
    fn typed_config_derives_and_validates_key() {
        let config = config().expect("config");
        assert_eq!(config.encryption_key.len(), 32);
        config.validate().expect("validate");
    }

    #[test]
    fn rejects_unbounded_or_unsafe_values() {
        assert!(AcmeConfig::new(
            "http://acme.invalid/directory".to_string(),
            "admin@example.com".to_string(),
            30,
            None,
            b"0123456789abcdef0123456789abcdef",
            false,
            true,
        )
        .is_err());
        assert!(AcmeConfig::new(
            "https://acme.example/directory".to_string(),
            "invalid email".to_string(),
            0,
            None,
            b"short",
            false,
            true,
        )
        .is_err());
    }
}
