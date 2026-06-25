use std::path::Path;
use std::time::Duration;

use instant_acme::{
    Account, AuthorizationStatus, ChallengeType, Identifier, NewAccount, NewOrder, OrderStatus,
    RetryPolicy,
};

use crate::challenge_store::ChallengeStore;
use crate::model::IssuedCertificateMaterial;
use crate::self_signed::fingerprint_sha256_hex;
use crate::{AcmeConfig, AcmeServiceError, AcmeServiceResult};

pub async fn issue_lets_encrypt(
    config: &AcmeConfig,
    challenge_store: &ChallengeStore,
    hostname: &str,
    cert_name: &str,
    cert_root: &str,
) -> AcmeServiceResult<IssuedCertificateMaterial> {
    let webroot = config.webroot.as_deref().map(Path::new).ok_or_else(|| {
        AcmeServiceError::config(
            "SDKWORK_WEB_ACME_WEBROOT is required for Let's Encrypt HTTP-01 issuance",
        )
    })?;

    let (account, _credentials) = Account::builder()
        .map_err(|error| AcmeServiceError::provider(error.to_string()))?
        .create(
            &NewAccount {
                contact: &[&format!("mailto:{}", config.contact_email)],
                terms_of_service_agreed: true,
                only_return_existing: false,
            },
            config.directory_url.clone(),
            None,
        )
        .await
        .map_err(|error| AcmeServiceError::provider(error.to_string()))?;

    let identifiers = [Identifier::Dns(hostname.to_string())];
    let mut order = account
        .new_order(&NewOrder::new(&identifiers))
        .await
        .map_err(|error| AcmeServiceError::provider(error.to_string()))?;

    let mut authorizations = order.authorizations();
    while let Some(result) = authorizations.next().await {
        let mut authz = result.map_err(|error| AcmeServiceError::provider(error.to_string()))?;
        if authz.status == AuthorizationStatus::Valid {
            continue;
        }

        let mut challenge = authz
            .challenge(ChallengeType::Http01)
            .ok_or_else(|| AcmeServiceError::provider("HTTP-01 challenge unavailable"))?;

        let token = challenge.token.clone();
        let key_auth = challenge.key_authorization().as_str().to_string();
        challenge_store.register(Some(webroot), &token, &key_auth)?;

        challenge
            .set_ready()
            .await
            .map_err(|error| AcmeServiceError::provider(error.to_string()))?;
    }

    let policy = RetryPolicy::default().timeout(Duration::from_secs(120));
    let status = order
        .poll_ready(&policy)
        .await
        .map_err(|error| AcmeServiceError::provider(error.to_string()))?;
    if status != OrderStatus::Ready {
        return Err(AcmeServiceError::provider(format!(
            "acme order not ready: {status:?}"
        )));
    }

    let private_key_pem = order
        .finalize()
        .await
        .map_err(|error| AcmeServiceError::provider(error.to_string()))?;
    let cert_chain_pem = order
        .poll_certificate(&policy)
        .await
        .map_err(|error| AcmeServiceError::provider(error.to_string()))?;

    let (not_before, not_after) = parse_certificate_validity(&cert_chain_pem)?;
    let fingerprint = fingerprint_sha256_hex(cert_chain_pem.as_bytes());
    let cert_dir = format!("{cert_root}/{cert_name}");
    let cert_path = format!("{cert_dir}/fullchain.pem");
    let key_path = format!("{cert_dir}/privkey.pem");

    Ok(IssuedCertificateMaterial {
        cert_name: cert_name.to_string(),
        cert_type: 1,
        issuer: if config.use_production {
            "Let's Encrypt".to_string()
        } else {
            "Let's Encrypt Staging".to_string()
        },
        subject: hostname.to_string(),
        san_list: hostname.to_string(),
        fingerprint,
        cert_pem: cert_chain_pem.clone(),
        private_key_pem,
        chain_pem: Some(cert_chain_pem),
        not_before,
        not_after,
        cert_path,
        key_path,
        chain_path: None,
    })
}

fn parse_certificate_validity(pem_chain: &str) -> AcmeServiceResult<(String, String)> {
    use x509_parser::pem::parse_x509_pem;

    let (_, pem) = parse_x509_pem(pem_chain.as_bytes())
        .map_err(|error| AcmeServiceError::Internal(error.to_string()))?;
    let cert = pem
        .parse_x509()
        .map_err(|error| AcmeServiceError::Internal(error.to_string()))?;
    let not_before = cert
        .validity()
        .not_before
        .to_rfc2822()
        .map_err(|error| AcmeServiceError::Internal(error))?;
    let not_after = cert
        .validity()
        .not_after
        .to_rfc2822()
        .map_err(|error| AcmeServiceError::Internal(error))?;
    Ok((not_before, not_after))
}
