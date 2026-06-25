use chrono::{Duration, Utc};
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};
use sha2::{Digest, Sha256};

use crate::model::IssuedCertificateMaterial;
use crate::{AcmeServiceError, AcmeServiceResult};

pub fn issue_self_signed(
    hostname: &str,
    cert_name: &str,
    cert_root: &str,
) -> AcmeServiceResult<IssuedCertificateMaterial> {
    let mut params = CertificateParams::new(vec![hostname.to_string()])
        .map_err(|error| AcmeServiceError::Internal(error.to_string()))?;

    params.distinguished_name = DistinguishedName::new();
    params.distinguished_name.push(DnType::CommonName, hostname);

    let not_before = Utc::now();
    let not_after = not_before + Duration::days(825);

    let key_pair =
        KeyPair::generate().map_err(|error| AcmeServiceError::Internal(error.to_string()))?;
    let cert = params
        .self_signed(&key_pair)
        .map_err(|error| AcmeServiceError::Internal(error.to_string()))?;

    let cert_pem = cert.pem();
    let private_key_pem = key_pair.serialize_pem();
    let fingerprint = fingerprint_sha256_hex(cert_pem.as_bytes());

    let cert_dir = format!("{cert_root}/{cert_name}");
    let cert_path = format!("{cert_dir}/fullchain.pem");
    let key_path = format!("{cert_dir}/privkey.pem");

    Ok(IssuedCertificateMaterial {
        cert_name: cert_name.to_string(),
        cert_type: 3,
        issuer: "SDKWork Web Server Self-Signed".to_string(),
        subject: hostname.to_string(),
        san_list: hostname.to_string(),
        fingerprint,
        cert_pem,
        private_key_pem,
        chain_pem: None,
        not_before: not_before.to_rfc3339(),
        not_after: not_after.to_rfc3339(),
        cert_path,
        key_path,
        chain_path: None,
    })
}

pub fn fingerprint_sha256_hex(pem_or_der: &[u8]) -> String {
    let digest = Sha256::digest(pem_or_der);
    hex::encode(digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_signed_material_is_pem_encoded() {
        let material =
            issue_self_signed("dev.localhost", "dev-localhost", "/tmp/certs/live").expect("issue");
        assert!(material.cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(material.private_key_pem.contains("BEGIN"));
    }
}
