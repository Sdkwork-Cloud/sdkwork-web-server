use std::sync::Arc;

use crate::challenge_store::ChallengeStore;
use crate::config::AcmeConfig;
use crate::encrypt::{decrypt_secret, encrypt_secret};
use crate::lets_encrypt::issue_lets_encrypt;
use crate::model::IssuedCertificateMaterial;
use crate::self_signed::issue_self_signed;
use crate::{AcmeServiceError, AcmeServiceResult};

pub struct CertificateIssuer {
    config: AcmeConfig,
    challenge_store: Arc<ChallengeStore>,
    cert_root: String,
}

impl CertificateIssuer {
    pub fn from_env() -> AcmeServiceResult<Self> {
        let config = AcmeConfig::from_env()?;
        let cert_root = std::env::var("SDKWORK_WEB_CERT_LIVE_ROOT")
            .unwrap_or_else(|_| "/opt/certs/letsencrypt/live".to_string());
        Ok(Self {
            config,
            challenge_store: Arc::new(ChallengeStore::default()),
            cert_root,
        })
    }

    pub fn challenge_store(&self) -> Arc<ChallengeStore> {
        self.challenge_store.clone()
    }

    pub fn cert_root(&self) -> &str {
        &self.cert_root
    }

    pub fn renew_before_days(&self) -> u32 {
        self.config.renew_before_days
    }

    pub async fn issue(
        &self,
        cert_type: i32,
        hostname: &str,
        cert_name: &str,
    ) -> AcmeServiceResult<IssuedCertificateMaterial> {
        match cert_type {
            1 => {
                issue_lets_encrypt(
                    &self.config,
                    self.challenge_store.as_ref(),
                    hostname,
                    cert_name,
                    &self.cert_root,
                )
                .await
            }
            3 => issue_self_signed(hostname, cert_name, &self.cert_root),
            other => Err(AcmeServiceError::validation(format!(
                "unsupported certType {other}; supported: 1 (Let's Encrypt), 3 (self-signed)"
            ))),
        }
    }

    pub fn encrypt_private_key(&self, private_key_pem: &str) -> AcmeServiceResult<String> {
        encrypt_secret(&self.config.encryption_key, private_key_pem.as_bytes())
    }

    pub fn decrypt_private_key(&self, encoded: &str) -> AcmeServiceResult<String> {
        let bytes = decrypt_secret(&self.config.encryption_key, encoded)?;
        String::from_utf8(bytes).map_err(|error| AcmeServiceError::Encryption(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn issues_self_signed_certificate() {
        std::env::set_var(
            "SDKWORK_WEB_CERT_ENCRYPTION_KEY",
            "test-encryption-key-for-acme-service",
        );
        let issuer = CertificateIssuer::from_env().expect("config");
        let material = issuer
            .issue(3, "dev.localhost", "dev-localhost")
            .await
            .expect("issue");
        assert_eq!(material.cert_type, 3);
        assert!(material.cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(material.private_key_pem.contains("BEGIN PRIVATE KEY"));
    }
}
