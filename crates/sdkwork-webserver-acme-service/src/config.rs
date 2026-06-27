use sdkwork_utils_rust::derive_aes_256_key;

use crate::{AcmeServiceError, AcmeServiceResult};

/// HKDF info 上下文绑定，将派生密钥绑定到 ACME 证书加密用途，防止跨用途密钥复用。
const CERT_ENCRYPTION_KEY_INFO: &[u8] = b"sdkwork-web-acme-cert-encryption";

/// Runtime ACME configuration loaded from environment.
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
    pub fn from_env() -> AcmeServiceResult<Self> {
        let use_production = matches!(
            std::env::var("SDKWORK_WEB_ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string())
                .to_ascii_lowercase()
                .as_str(),
            "production" | "prod"
        );

        let directory_url = std::env::var("SDKWORK_WEB_ACME_DIRECTORY_URL").unwrap_or_else(|_| {
            if use_production {
                "https://acme-v02.api.letsencrypt.org/directory".to_string()
            } else {
                "https://acme-staging-v02.api.letsencrypt.org/directory".to_string()
            }
        });

        let contact_email = std::env::var("SDKWORK_WEB_ACME_CONTACT_EMAIL")
            .unwrap_or_else(|_| "admin@localhost".to_string());

        let renew_before_days = std::env::var("SDKWORK_WEB_CERT_RENEW_BEFORE_DAYS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(30);

        let webroot = std::env::var("SDKWORK_WEB_ACME_WEBROOT").ok();

        let encryption_key = load_encryption_key()?;

        Ok(Self {
            directory_url,
            contact_email,
            renew_before_days,
            webroot,
            encryption_key,
            use_production,
        })
    }
}

fn load_encryption_key() -> AcmeServiceResult<Vec<u8>> {
    let use_production = matches!(
        std::env::var("SDKWORK_WEB_ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .to_ascii_lowercase()
            .as_str(),
        "production" | "prod"
    );

    let raw = match std::env::var("SDKWORK_WEB_CERT_ENCRYPTION_KEY") {
        Ok(value) => value,
        Err(_) if !use_production => {
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

    // 使用 HKDF-SHA256 派生 32 字节 AES-256 密钥，替代裸 SHA-256 摘要。
    // HKDF 提供标准密钥派生流程（RFC 5869），支持任意长度输入，
    // 并通过 info 上下文绑定将密钥隔离到 ACME 证书加密用途。
    let derived = derive_aes_256_key(
        raw.as_bytes(),
        b"sdkwork-web-acme",
        CERT_ENCRYPTION_KEY_INFO,
    );
    Ok(derived.to_vec())
}
