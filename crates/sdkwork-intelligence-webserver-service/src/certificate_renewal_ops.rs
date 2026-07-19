//! Certificate renewal scan and re-issuance for autoRenew certificates.

use sdkwork_webserver_contract::{
    CertificateIssueUpdate, CertificateRenewalCandidate, CertificateRenewalCycleReport,
    CertificateResponse, WebServiceError, WebServiceResult,
};

use crate::{AuditLogWrite, WebService};

impl WebService {
    pub async fn run_certificate_renewal_cycle(
        &self,
    ) -> WebServiceResult<CertificateRenewalCycleReport> {
        let renew_before_days = self.certificate_issuer.renew_before_days();
        let candidates = self
            .repository
            .list_certificates_due_for_renewal(renew_before_days, 50)
            .await?;

        let mut report = CertificateRenewalCycleReport {
            scanned: candidates.len(),
            renewed: 0,
            failed: 0,
        };

        for candidate in candidates {
            match self.renew_certificate(&candidate).await {
                Ok(_) => report.renewed += 1,
                Err(error) => {
                    report.failed += 1;
                    tracing::warn!(
                        tenant_id = candidate.tenant_id,
                        certificate_id = %candidate.certificate_id,
                        error = %error,
                        "certificate renewal failed"
                    );
                }
            }
        }

        Ok(report)
    }

    async fn renew_certificate(
        &self,
        candidate: &CertificateRenewalCandidate,
    ) -> WebServiceResult<CertificateResponse> {
        if !candidate.auto_renew {
            return Err(WebServiceError::validation("auto_renew is disabled"));
        }

        if !matches!(candidate.cert_type, 1 | 3) {
            return Err(WebServiceError::validation(format!(
                "certType {} is not eligible for automatic renewal",
                candidate.cert_type
            )));
        }

        if !self
            .repository
            .mark_certificate_renewing(candidate.tenant_id, &candidate.certificate_id)
            .await?
        {
            return Err(WebServiceError::conflict(
                "certificate renewal already in progress",
            ));
        }

        let issue_result = self
            .certificate_issuer
            .issue(
                candidate.cert_type,
                &candidate.hostname,
                &candidate.cert_name,
            )
            .await;

        let material = match issue_result {
            Ok(material) => material,
            Err(error) => {
                let _ = self
                    .repository
                    .fail_certificate_renewal(
                        candidate.tenant_id,
                        &candidate.certificate_id,
                        &error.to_string(),
                    )
                    .await;
                return Err(WebServiceError::Internal(error.to_string()));
            }
        };

        self.persist_issued_certificate(
            candidate.tenant_id,
            &candidate.certificate_id,
            candidate.auto_renew,
            material,
            "certificates.renew",
        )
        .await
    }

    pub(crate) async fn persist_issued_certificate(
        &self,
        tenant_id: i64,
        certificate_id: &str,
        auto_renew: bool,
        material: sdkwork_webserver_acme_service::IssuedCertificateMaterial,
        audit_action: &str,
    ) -> WebServiceResult<CertificateResponse> {
        let encrypted_private_key = self
            .certificate_issuer
            .encrypt_private_key(&material.private_key_pem)
            .map_err(|error| WebServiceError::Internal(error.to_string()))?;

        let write_result = self
            .edge_runtime
            .write_certificate_bundle_async(&material)
            .await;
        if let Err(error) = write_result {
            if audit_action == "certificates.issue" {
                let _ = self
                    .repository
                    .fail_certificate(tenant_id, certificate_id, &error.to_string())
                    .await;
            } else {
                let _ = self
                    .repository
                    .fail_certificate_renewal(tenant_id, certificate_id, &error.to_string())
                    .await;
            }
            return Err(WebServiceError::Internal(error.to_string()));
        }

        let update = CertificateIssueUpdate {
            cert_name: material.cert_name,
            cert_type: material.cert_type,
            issuer: material.issuer,
            subject: material.subject,
            san_list: material.san_list,
            fingerprint: material.fingerprint,
            cert_path: material.cert_path,
            key_path: material.key_path,
            chain_path: material.chain_path,
            not_before: material.not_before,
            not_after: material.not_after,
            auto_renew,
            cert_pem: material.cert_pem.clone(),
            chain_pem: material.chain_pem.clone(),
            encrypted_private_key,
        };

        let response = self
            .repository
            .finalize_certificate(tenant_id, certificate_id, &update)
            .await?;

        let _ = self
            .repository
            .insert_audit_log(AuditLogWrite {
                tenant_id,
                organization_id: 0,
                operator_id: 0,
                action: audit_action,
                target_type: "certificate",
                target_id: None,
                target_uuid: Some(&response.id),
            })
            .await;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    fn certificate_due_for_renewal(not_after: &str, renew_before_days: u32) -> bool {
        use chrono::DateTime;
        let Ok(not_after) = DateTime::parse_from_rfc3339(not_after) else {
            return false;
        };
        let threshold = Utc::now() + Duration::days(i64::from(renew_before_days));
        not_after.with_timezone(&Utc) <= threshold
    }

    #[test]
    fn due_when_expiry_within_renew_window() {
        let soon = (Utc::now() + Duration::days(10)).to_rfc3339();
        assert!(certificate_due_for_renewal(&soon, 30));
    }

    #[test]
    fn not_due_when_expiry_far_future() {
        let later = (Utc::now() + Duration::days(120)).to_rfc3339();
        assert!(!certificate_due_for_renewal(&later, 30));
    }
}
