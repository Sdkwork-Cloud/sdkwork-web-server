use chrono::{Duration, TimeZone, Utc};
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};
use sdkwork_utils_rust::crypto::sha256_hash;
use time::OffsetDateTime;
use x509_parser::pem::parse_x509_pem;

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

    let now = Utc::now();
    let not_before = now - Duration::minutes(5);
    let not_after = not_before + Duration::days(825);
    params.not_before = OffsetDateTime::from_unix_timestamp(not_before.timestamp())
        .map_err(|error| AcmeServiceError::Internal(error.to_string()))?;
    params.not_after = OffsetDateTime::from_unix_timestamp(not_after.timestamp())
        .map_err(|error| AcmeServiceError::Internal(error.to_string()))?;

    let key_pair =
        KeyPair::generate().map_err(|error| AcmeServiceError::Internal(error.to_string()))?;
    let cert = params
        .self_signed(&key_pair)
        .map_err(|error| AcmeServiceError::Internal(error.to_string()))?;

    let cert_pem = cert.pem();
    let private_key_pem = key_pair.serialize_pem();
    let (not_before, not_after, fingerprint) = certificate_evidence_from_pem(&cert_pem)?;
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
        not_before,
        not_after,
        cert_path,
        key_path,
        chain_path: None,
    })
}

pub(crate) fn certificate_evidence_from_pem(
    pem_chain: &str,
) -> AcmeServiceResult<(String, String, String)> {
    let (_, pem) = parse_x509_pem(pem_chain.as_bytes())
        .map_err(|error| AcmeServiceError::Internal(error.to_string()))?;
    if pem.label != "CERTIFICATE" {
        return Err(AcmeServiceError::Internal(
            "first PEM block is not a certificate".to_string(),
        ));
    }
    let cert = pem
        .parse_x509()
        .map_err(|error| AcmeServiceError::Internal(error.to_string()))?;
    let not_before = timestamp_to_rfc3339(cert.validity().not_before.timestamp())?;
    let not_after = timestamp_to_rfc3339(cert.validity().not_after.timestamp())?;
    let fingerprint = fingerprint_sha256_hex(&pem.contents);
    Ok((not_before, not_after, fingerprint))
}

fn timestamp_to_rfc3339(timestamp: i64) -> AcmeServiceResult<String> {
    Utc.timestamp_opt(timestamp, 0)
        .single()
        .map(|value| value.to_rfc3339())
        .ok_or_else(|| AcmeServiceError::Internal("certificate timestamp is invalid".to_string()))
}

pub fn fingerprint_sha256_hex(der: &[u8]) -> String {
    sha256_hash(der)
}

#[cfg(test)]
mod tests {
    use super::*;
    use x509_parser::extensions::GeneralName;

    #[test]
    fn self_signed_material_matches_actual_leaf_evidence() {
        let material =
            issue_self_signed("dev.localhost", "dev-localhost", "/tmp/certs/live").expect("issue");
        let (_, pem) = parse_x509_pem(material.cert_pem.as_bytes()).expect("PEM");
        let cert = pem.parse_x509().expect("X.509");
        let sans = cert
            .subject_alternative_name()
            .expect("SAN extension")
            .expect("SAN present");
        assert!(sans
            .value
            .general_names
            .iter()
            .any(|name| matches!(name, GeneralName::DNSName(value) if *value == "dev.localhost")));

        let (not_before, not_after, fingerprint) =
            certificate_evidence_from_pem(&material.cert_pem).expect("evidence");
        assert_eq!(material.not_before, not_before);
        assert_eq!(material.not_after, not_after);
        assert_eq!(material.fingerprint, fingerprint);
        assert_eq!(fingerprint, fingerprint_sha256_hex(&pem.contents));
        assert!(chrono::DateTime::parse_from_rfc3339(&material.not_before).is_ok());
        assert!(chrono::DateTime::parse_from_rfc3339(&material.not_after).is_ok());
        assert!(material.private_key_pem.contains("BEGIN PRIVATE KEY"));
    }
}
