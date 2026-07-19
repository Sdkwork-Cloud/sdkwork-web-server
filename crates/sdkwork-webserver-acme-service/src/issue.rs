use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

use crate::challenge_store::ChallengeStore;
use crate::config::AcmeConfig;
use crate::encrypt::{decrypt_secret, encrypt_secret};
use crate::lets_encrypt::issue_lets_encrypt;
use crate::model::IssuedCertificateMaterial;
use crate::self_signed::issue_self_signed;
use crate::{AcmeServiceError, AcmeServiceResult};
use crate::{
    DEFAULT_ACME_OPERATION_TIMEOUT_MS, MAX_ACME_OPERATION_TIMEOUT_MS, MIN_ACME_OPERATION_TIMEOUT_MS,
};

const MAX_CONCURRENT_CERTIFICATE_ISSUANCE: usize = 8;

pub struct CertificateIssuer {
    config: AcmeConfig,
    challenge_store: Arc<ChallengeStore>,
    cert_root: String,
    operation_timeout: Duration,
    admission: Semaphore,
}

impl CertificateIssuer {
    pub fn new(config: AcmeConfig, cert_root: impl Into<String>) -> AcmeServiceResult<Self> {
        Self::new_with_operation_timeout_ms(config, cert_root, DEFAULT_ACME_OPERATION_TIMEOUT_MS)
    }

    pub fn new_with_operation_timeout_ms(
        config: AcmeConfig,
        cert_root: impl Into<String>,
        operation_timeout_ms: u64,
    ) -> AcmeServiceResult<Self> {
        config.validate()?;
        let cert_root = cert_root.into();
        if cert_root.is_empty()
            || cert_root.len() > 4_096
            || cert_root
                .bytes()
                .any(|byte| byte == 0 || byte.is_ascii_control())
        {
            return Err(AcmeServiceError::config(
                "certificate live root must contain 1..4096 safe path bytes",
            ));
        }
        if !(MIN_ACME_OPERATION_TIMEOUT_MS..=MAX_ACME_OPERATION_TIMEOUT_MS)
            .contains(&operation_timeout_ms)
        {
            return Err(AcmeServiceError::config(format!(
                "ACME operation timeout must be between {MIN_ACME_OPERATION_TIMEOUT_MS} and {MAX_ACME_OPERATION_TIMEOUT_MS} ms"
            )));
        }
        Ok(Self {
            config,
            challenge_store: Arc::new(ChallengeStore::default()),
            cert_root,
            operation_timeout: Duration::from_millis(operation_timeout_ms),
            admission: Semaphore::new(MAX_CONCURRENT_CERTIFICATE_ISSUANCE),
        })
    }

    /// Compatibility loader. Deployable runtimes should inject typed configuration.
    pub fn from_env() -> AcmeServiceResult<Self> {
        let config = AcmeConfig::from_env()?;
        let cert_root = std::env::var("SDKWORK_WEB_CERT_LIVE_ROOT")
            .unwrap_or_else(|_| "/opt/certs/letsencrypt/live".to_string());
        let operation_timeout_ms = std::env::var("SDKWORK_WEB_ACME_OPERATION_TIMEOUT_MS")
            .map(|value| {
                value.parse::<u64>().map_err(|error| {
                    AcmeServiceError::config(format!(
                        "invalid SDKWORK_WEB_ACME_OPERATION_TIMEOUT_MS: {error}"
                    ))
                })
            })
            .unwrap_or(Ok(DEFAULT_ACME_OPERATION_TIMEOUT_MS))?;
        Self::new_with_operation_timeout_ms(config, cert_root, operation_timeout_ms)
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
        validate_hostname(hostname)?;
        validate_certificate_name(cert_name)?;
        let _permit = self.admission.try_acquire().map_err(|_| {
            AcmeServiceError::provider(format!(
                "certificate issuance capacity exhausted; maximum concurrent operations: {MAX_CONCURRENT_CERTIFICATE_ISSUANCE}"
            ))
        })?;
        match cert_type {
            1 => {
                issue_lets_encrypt(
                    &self.config,
                    self.challenge_store.as_ref(),
                    hostname,
                    cert_name,
                    &self.cert_root,
                    self.operation_timeout,
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

fn validate_hostname(hostname: &str) -> AcmeServiceResult<()> {
    if hostname.is_empty()
        || hostname.len() > 253
        || hostname.starts_with('.')
        || hostname.ends_with('.')
        || hostname.split('.').any(|label| {
            label.is_empty()
                || label.len() > 63
                || label.starts_with('-')
                || label.ends_with('-')
                || !label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
    {
        return Err(AcmeServiceError::validation(
            "hostname must be a safe ASCII DNS name",
        ));
    }
    Ok(())
}

fn validate_certificate_name(cert_name: &str) -> AcmeServiceResult<()> {
    if cert_name.is_empty()
        || cert_name.len() > 253
        || matches!(cert_name, "." | "..")
        || cert_name.starts_with('.')
        || cert_name.ends_with('.')
        || cert_name.contains("..")
        || !cert_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(AcmeServiceError::validation(
            "certificate name must contain 1..253 safe ASCII name bytes",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn issues_self_signed_certificate() {
        let config = AcmeConfig::new(
            "https://acme-staging-v02.api.letsencrypt.org/directory".to_string(),
            "admin@example.com".to_string(),
            30,
            None,
            b"test-encryption-key-for-acme-service",
            false,
            false,
        )
        .expect("config");
        let issuer = CertificateIssuer::new(config, "/tmp/certs/live").expect("issuer");
        let material = issuer
            .issue(3, "dev.localhost", "dev-localhost")
            .await
            .expect("issue");
        assert_eq!(material.cert_type, 3);
        assert!(material.cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(material.private_key_pem.contains("BEGIN PRIVATE KEY"));
    }

    #[test]
    fn rejects_unbounded_operation_timeout() {
        let config = AcmeConfig::new(
            "https://acme-staging-v02.api.letsencrypt.org/directory".to_string(),
            "admin@example.com".to_string(),
            30,
            None,
            b"test-encryption-key-for-acme-service",
            false,
            false,
        )
        .expect("config");
        assert!(
            CertificateIssuer::new_with_operation_timeout_ms(config, "/tmp/certs", 9_999).is_err()
        );
    }

    #[tokio::test]
    async fn rejects_unsafe_hostname_and_certificate_name() {
        let config = AcmeConfig::new(
            "https://acme-staging-v02.api.letsencrypt.org/directory".to_string(),
            "admin@example.com".to_string(),
            30,
            None,
            b"test-encryption-key-for-acme-service",
            false,
            false,
        )
        .expect("config");
        let issuer = CertificateIssuer::new(config, "/tmp/certs/live").expect("issuer");
        assert!(issuer.issue(3, "../escape", "safe-name").await.is_err());
        assert!(issuer.issue(3, "dev.localhost", "../escape").await.is_err());
    }

    #[tokio::test]
    async fn issuance_admission_has_no_waiter_queue() {
        let config = AcmeConfig::new(
            "https://acme-staging-v02.api.letsencrypt.org/directory".to_string(),
            "admin@example.com".to_string(),
            30,
            None,
            b"test-encryption-key-for-acme-service",
            false,
            false,
        )
        .expect("config");
        let issuer = CertificateIssuer::new(config, "/tmp/certs/live").expect("issuer");
        let permits = (0..MAX_CONCURRENT_CERTIFICATE_ISSUANCE)
            .map(|_| issuer.admission.try_acquire().expect("permit"))
            .collect::<Vec<_>>();
        let error = issuer
            .issue(3, "dev.localhost", "dev-localhost")
            .await
            .expect_err("capacity must fail closed");
        assert!(error.to_string().contains("capacity exhausted"));
        drop(permits);
        issuer
            .issue(3, "dev.localhost", "dev-localhost")
            .await
            .expect("capacity recovers");
    }
}
