use std::{
    collections::HashMap,
    io::{self, Cursor},
    path::Path,
    sync::Arc,
};

use axum_server::tls_rustls::RustlsConfig;
use rustls::{
    pki_types::PrivateKeyDer,
    server::{ClientHello, ResolvesServerCert},
    sign::CertifiedKey,
    ServerConfig,
};
use rustls_pemfile::Item;
use sdkwork_webserver_core::{normalize_server_name, server_name_covers, ListenerConfig};
use x509_parser::prelude::{FromDer, GeneralName, X509Certificate};

use super::{
    runtime::{read_bounded_tls_material, RuntimeGeneration},
    DataPlaneError,
};

#[derive(Debug)]
struct SniCertificateResolver {
    exact: HashMap<String, Arc<CertifiedKey>>,
    wildcards: HashMap<String, Arc<CertifiedKey>>,
}

impl ResolvesServerCert for SniCertificateResolver {
    fn resolve(&self, client_hello: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        let server_name = normalize_server_name(client_hello.server_name()?)?;
        if let Some(certified_key) = self.exact.get(&server_name) {
            return Some(certified_key.clone());
        }
        self.wildcards
            .get(wildcard_lookup_suffix(&server_name)?)
            .cloned()
    }
}

pub(crate) fn build_tls_config(
    generation: &Arc<RuntimeGeneration>,
    listener: &ListenerConfig,
) -> Result<Option<RustlsConfig>, DataPlaneError> {
    let Some(policy_id) = &listener.tls_policy_ref else {
        return Ok(None);
    };
    let policy =
        generation
            .app
            .tls_policy(policy_id)
            .ok_or_else(|| DataPlaneError::MissingTlsPolicy {
                listener_id: listener.id.clone(),
                policy_id: policy_id.clone(),
            })?;

    install_crypto_provider()?;
    let provider = rustls::crypto::CryptoProvider::get_default()
        .expect("the Rustls crypto provider was installed before certificate parsing");
    let mut exact = HashMap::new();
    let mut wildcards = HashMap::new();
    for certificate_ref in policy.certificate_refs() {
        let certificate = generation.app.certificate(certificate_ref).ok_or_else(|| {
            DataPlaneError::MissingCertificate {
                policy_id: policy.id.clone(),
                certificate_id: certificate_ref.to_owned(),
            }
        })?;
        let (certificate_file, private_key_file) = generation
            .app
            .certificate_paths(&certificate.id)
            .ok_or_else(|| DataPlaneError::MissingCertificateFiles {
                certificate_id: certificate.id.clone(),
            })?;
        let certified_key = Arc::new(load_certified_key(
            certificate_file,
            private_key_file,
            &certificate.server_names,
            provider,
        )?);
        for server_name in &certificate.server_names {
            let normalized = normalize_server_name(server_name)
                .expect("semantic validation guarantees normalized certificate server names");
            if let Some(suffix) = normalized.strip_prefix("*.") {
                if wildcards
                    .insert(suffix.to_owned(), certified_key.clone())
                    .is_some()
                {
                    return Err(DataPlaneError::AmbiguousTlsServerName {
                        policy_id: policy.id.clone(),
                        server_name: normalized,
                    });
                }
            } else if exact
                .insert(normalized.clone(), certified_key.clone())
                .is_some()
            {
                return Err(DataPlaneError::AmbiguousTlsServerName {
                    policy_id: policy.id.clone(),
                    server_name: normalized,
                });
            }
        }
    }
    let resolver = SniCertificateResolver { exact, wildcards };
    let mut server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(resolver));
    server_config.alpn_protocols = policy
        .alpn
        .iter()
        .map(|protocol| protocol.as_bytes().to_vec())
        .collect();
    Ok(Some(RustlsConfig::from_config(Arc::new(server_config))))
}

fn install_crypto_provider() -> Result<(), DataPlaneError> {
    if rustls::crypto::CryptoProvider::get_default().is_some() {
        return Ok(());
    }
    if rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .is_err()
        && rustls::crypto::CryptoProvider::get_default().is_none()
    {
        return Err(DataPlaneError::TlsCryptoProvider);
    }
    Ok(())
}

fn load_certified_key(
    certificate_file: &Path,
    private_key_file: &Path,
    declared_server_names: &[String],
    provider: &rustls::crypto::CryptoProvider,
) -> Result<CertifiedKey, DataPlaneError> {
    let certificate_pem = read_bounded_tls_material(certificate_file)?;
    let private_key_pem = read_bounded_tls_material(private_key_file)?;
    let certificate_chain = parse_certificate_chain(&certificate_pem)
        .map_err(|source| tls_files_error(certificate_file, private_key_file, source))?;
    validate_leaf_certificate(certificate_chain[0].as_ref(), declared_server_names)
        .map_err(|source| tls_files_error(certificate_file, private_key_file, source))?;
    let private_key = parse_single_private_key(&private_key_pem)
        .map_err(|source| tls_files_error(certificate_file, private_key_file, source))?;
    CertifiedKey::from_der(certificate_chain, private_key, provider).map_err(|source| {
        tls_files_error(
            certificate_file,
            private_key_file,
            io::Error::new(io::ErrorKind::InvalidData, source),
        )
    })
}

fn parse_certificate_chain(
    bytes: &[u8],
) -> Result<Vec<rustls::pki_types::CertificateDer<'static>>, io::Error> {
    let mut certificates = Vec::new();
    for item in rustls_pemfile::read_all(&mut Cursor::new(bytes)) {
        match item? {
            Item::X509Certificate(certificate) => certificates.push(certificate),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "certificate file contains a non-certificate PEM item",
                ))
            }
        }
    }
    if certificates.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "certificate chain is empty",
        ));
    }
    Ok(certificates)
}

fn validate_leaf_certificate(
    certificate_der: &[u8],
    declared_server_names: &[String],
) -> Result<(), io::Error> {
    let (remaining, certificate) =
        X509Certificate::from_der(certificate_der).map_err(|source| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("cannot parse leaf X.509 certificate: {source}"),
            )
        })?;
    if !remaining.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "leaf X.509 certificate has trailing DER data",
        ));
    }
    if !certificate.validity().is_valid() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "leaf X.509 certificate is expired or not yet valid",
        ));
    }
    let subject_alternative_name = certificate
        .subject_alternative_name()
        .map_err(|source| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid subjectAltName extension: {source}"),
            )
        })?
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "leaf X.509 certificate has no subjectAltName extension",
            )
        })?;
    let certificate_dns_names = subject_alternative_name
        .value
        .general_names
        .iter()
        .filter_map(|general_name| match general_name {
            GeneralName::DNSName(name) => Some(*name),
            _ => None,
        })
        .collect::<Vec<_>>();
    for declared_server_name in declared_server_names {
        if !certificate_dns_names
            .iter()
            .any(|certificate_name| server_name_covers(certificate_name, declared_server_name))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "leaf X.509 subjectAltName does not cover declared server name {declared_server_name}"
                ),
            ));
        }
    }
    Ok(())
}

fn parse_single_private_key(bytes: &[u8]) -> Result<PrivateKeyDer<'static>, io::Error> {
    let mut keys = rustls_pemfile::read_all(&mut Cursor::new(bytes))
        .map(|item| {
            item.and_then(|item| match item {
                Item::Pkcs1Key(key) => Ok(Some(PrivateKeyDer::Pkcs1(key))),
                Item::Pkcs8Key(key) => Ok(Some(PrivateKeyDer::Pkcs8(key))),
                Item::Sec1Key(key) => Ok(Some(PrivateKeyDer::Sec1(key))),
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "private-key file contains a non-key PEM item",
                )),
            })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten();
    let key = keys.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "private-key file contains no key",
        )
    })?;
    if keys.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "private-key file contains more than one key",
        ));
    }
    Ok(key)
}

fn tls_files_error(
    certificate_file: &Path,
    private_key_file: &Path,
    source: io::Error,
) -> DataPlaneError {
    DataPlaneError::TlsFiles {
        certificate_file: certificate_file.to_path_buf(),
        private_key_file: private_key_file.to_path_buf(),
        source,
    }
}

fn wildcard_lookup_suffix(server_name: &str) -> Option<&str> {
    server_name.split_once('.').map(|(_, suffix)| suffix)
}

#[cfg(test)]
mod tests {
    use rcgen::{date_time_ymd, CertificateParams, KeyPair};

    use super::{
        parse_certificate_chain, parse_single_private_key, validate_leaf_certificate,
        wildcard_lookup_suffix,
    };

    #[test]
    fn wildcard_matches_exactly_one_left_label() {
        assert_eq!(
            wildcard_lookup_suffix("www.example.test"),
            Some("example.test")
        );
        assert_eq!(wildcard_lookup_suffix("example.test"), Some("test"));
        assert_ne!(
            wildcard_lookup_suffix("a.b.example.test"),
            Some("example.test")
        );
    }

    #[test]
    fn leaf_certificate_must_cover_every_declared_server_name() {
        let params = CertificateParams::new(vec!["alpha.example.test".to_owned()])
            .expect("certificate parameters");
        let key = KeyPair::generate().expect("generate key");
        let certificate = params.self_signed(&key).expect("generate certificate");

        validate_leaf_certificate(
            certificate.der().as_ref(),
            &["alpha.example.test".to_owned()],
        )
        .expect("matching SAN must validate");
        let error = validate_leaf_certificate(
            certificate.der().as_ref(),
            &["beta.example.test".to_owned()],
        )
        .expect_err("mismatched SAN must fail");
        assert!(error.to_string().contains("does not cover"));
    }

    #[test]
    fn leaf_certificate_must_be_currently_valid() {
        let mut params = CertificateParams::new(vec!["expired.example.test".to_owned()])
            .expect("certificate parameters");
        params.not_before = date_time_ymd(2000, 1, 1);
        params.not_after = date_time_ymd(2001, 1, 1);
        let key = KeyPair::generate().expect("generate key");
        let certificate = params.self_signed(&key).expect("generate certificate");

        let error = validate_leaf_certificate(
            certificate.der().as_ref(),
            &["expired.example.test".to_owned()],
        )
        .expect_err("expired leaf must fail");
        assert!(error.to_string().contains("expired or not yet valid"));
    }

    #[test]
    fn empty_or_malformed_pem_material_is_rejected() {
        assert!(parse_certificate_chain(b"not a PEM certificate").is_err());
        assert!(parse_single_private_key(b"not a PEM private key").is_err());
    }
}
