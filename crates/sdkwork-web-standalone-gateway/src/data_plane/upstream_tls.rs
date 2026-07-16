use reqwest::{tls::Version, Certificate, ClientBuilder, Identity};
use sdkwork_webserver_core::{
    CompiledWebServerApp, TlsVersion, UpstreamConfig, UpstreamTlsTrustMode,
};

use super::{runtime::read_bounded_tls_material, DataPlaneError};

const MAX_CUSTOM_ROOT_CERTIFICATES: usize = 64;

pub(crate) fn configure_upstream_tls(
    mut builder: ClientBuilder,
    app: &CompiledWebServerApp,
    upstream: &UpstreamConfig,
) -> Result<ClientBuilder, DataPlaneError> {
    let Some(policy) = &upstream.tls else {
        return Ok(builder);
    };

    builder = builder
        .use_rustls_tls()
        .tls_built_in_root_certs(matches!(
            policy.trust_mode,
            UpstreamTlsTrustMode::System | UpstreamTlsTrustMode::SystemAndCustom
        ))
        .min_tls_version(reqwest_tls_version(policy.minimum_version))
        .max_tls_version(reqwest_tls_version(policy.maximum_version));

    let ca_paths = app
        .upstream_tls_ca_certificate_paths(&upstream.id)
        .expect("compilation resolves every configured upstream TLS policy");
    let mut root_count = 0usize;
    for path in ca_paths {
        let pem = read_bounded_tls_material(path)?;
        let certificates =
            Certificate::from_pem_bundle(&pem).map_err(|source| DataPlaneError::UpstreamTls {
                upstream_id: upstream.id.clone(),
                material: "CA certificate bundle",
                source,
            })?;
        if certificates.is_empty() {
            return Err(DataPlaneError::EmptyUpstreamCaBundle {
                upstream_id: upstream.id.clone(),
                path: path.clone(),
            });
        }
        root_count = root_count.saturating_add(certificates.len());
        if root_count > MAX_CUSTOM_ROOT_CERTIFICATES {
            return Err(DataPlaneError::TooManyUpstreamRootCertificates {
                upstream_id: upstream.id.clone(),
                actual: root_count,
                maximum: MAX_CUSTOM_ROOT_CERTIFICATES,
            });
        }
        for certificate in certificates {
            builder = builder.add_root_certificate(certificate);
        }
    }

    if let Some((certificate_path, private_key_path)) =
        app.upstream_tls_client_identity_paths(&upstream.id)
    {
        let certificate = read_bounded_tls_material(certificate_path)?;
        let private_key = read_bounded_tls_material(private_key_path)?;
        let mut identity_pem = Vec::with_capacity(
            certificate
                .len()
                .saturating_add(private_key.len())
                .saturating_add(2),
        );
        identity_pem.extend_from_slice(&certificate);
        if !identity_pem.ends_with(b"\n") {
            identity_pem.push(b'\n');
        }
        identity_pem.extend_from_slice(&private_key);
        let identity =
            Identity::from_pem(&identity_pem).map_err(|source| DataPlaneError::UpstreamTls {
                upstream_id: upstream.id.clone(),
                material: "client identity",
                source,
            })?;
        builder = builder.identity(identity);
    }

    Ok(builder)
}

fn reqwest_tls_version(version: TlsVersion) -> Version {
    match version {
        TlsVersion::Tls12 => Version::TLS_1_2,
        TlsVersion::Tls13 => Version::TLS_1_3,
    }
}
