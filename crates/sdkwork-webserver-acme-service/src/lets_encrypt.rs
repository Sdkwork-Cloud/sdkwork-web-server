use std::path::Path;
use std::time::Duration;

use instant_acme::{
    Account, AuthorizationStatus, ChallengeType, Identifier, NewAccount, NewOrder, OrderStatus,
    RetryPolicy,
};

use crate::challenge_store::ChallengeStore;
use crate::http_client::BoundedAcmeHttpClient;
use crate::model::IssuedCertificateMaterial;
use crate::self_signed::certificate_evidence_from_pem;
use crate::{AcmeConfig, AcmeServiceError, AcmeServiceResult};

const MAX_AUTHORIZATIONS_PER_ORDER: usize = 8;

pub async fn issue_lets_encrypt(
    config: &AcmeConfig,
    challenge_store: &ChallengeStore,
    hostname: &str,
    cert_name: &str,
    cert_root: &str,
    operation_timeout: Duration,
) -> AcmeServiceResult<IssuedCertificateMaterial> {
    match tokio::time::timeout(
        operation_timeout,
        issue_lets_encrypt_inner(
            config,
            challenge_store,
            hostname,
            cert_name,
            cert_root,
            operation_timeout,
        ),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => Err(AcmeServiceError::provider(format!(
            "ACME issuance timed out after {} ms",
            operation_timeout.as_millis()
        ))),
    }
}

async fn issue_lets_encrypt_inner(
    config: &AcmeConfig,
    challenge_store: &ChallengeStore,
    hostname: &str,
    cert_name: &str,
    cert_root: &str,
    operation_timeout: Duration,
) -> AcmeServiceResult<IssuedCertificateMaterial> {
    let webroot = config.webroot.as_deref().map(Path::new).ok_or_else(|| {
        AcmeServiceError::config(
            "SDKWORK_WEB_ACME_WEBROOT is required for Let's Encrypt HTTP-01 issuance",
        )
    })?;

    let contact = format!("mailto:{}", config.contact_email);
    let (account, _credentials) =
        Account::builder_with_http(Box::new(BoundedAcmeHttpClient::new()?))
            .create(
                &NewAccount {
                    contact: &[&contact],
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

    let mut challenge_leases = Vec::with_capacity(1);
    let mut authorization_count = 0_usize;
    let mut authorizations = order.authorizations();
    while let Some(result) = authorizations.next().await {
        authorization_count += 1;
        if authorization_count > MAX_AUTHORIZATIONS_PER_ORDER {
            return Err(AcmeServiceError::provider(format!(
                "ACME order exceeds {MAX_AUTHORIZATIONS_PER_ORDER} authorizations"
            )));
        }
        let mut authz = result.map_err(|error| AcmeServiceError::provider(error.to_string()))?;
        if authz.status == AuthorizationStatus::Valid {
            continue;
        }

        let mut challenge = authz
            .challenge(ChallengeType::Http01)
            .ok_or_else(|| AcmeServiceError::provider("HTTP-01 challenge unavailable"))?;
        let token = challenge.token.clone();
        let key_auth = challenge.key_authorization().as_str().to_string();
        let lease = challenge_store
            .register_scoped(Some(webroot), &token, &key_auth)
            .await?;

        challenge
            .set_ready()
            .await
            .map_err(|error| AcmeServiceError::provider(error.to_string()))?;
        challenge_leases.push(lease);
    }

    let retry_timeout = operation_timeout.min(Duration::from_secs(120));
    let policy = RetryPolicy::default().timeout(retry_timeout);
    let status = order
        .poll_ready(&policy)
        .await
        .map_err(|error| AcmeServiceError::provider(error.to_string()))?;
    if status != OrderStatus::Ready {
        return Err(AcmeServiceError::provider(format!(
            "ACME order not ready: {status:?}"
        )));
    }

    drop(challenge_leases);
    let private_key_pem = order
        .finalize()
        .await
        .map_err(|error| AcmeServiceError::provider(error.to_string()))?;
    let cert_chain_pem = order
        .poll_certificate(&policy)
        .await
        .map_err(|error| AcmeServiceError::provider(error.to_string()))?;

    let (not_before, not_after, fingerprint) = certificate_evidence_from_pem(&cert_chain_pem)?;
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
