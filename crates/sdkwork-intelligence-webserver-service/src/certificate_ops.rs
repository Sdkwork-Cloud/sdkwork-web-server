//! Certificate issuance orchestration using instant-acme/rcgen and edge runtime materialization.

use sdkwork_webserver_contract::{
    CertificateResponse, CreateCertificateRequest, WebAppRequestContext, WebServiceError,
    WebServiceResult,
};

use crate::WebService;

impl WebService {
    pub async fn issue_certificate(
        &self,
        context: &WebAppRequestContext,
        request: &CreateCertificateRequest,
    ) -> WebServiceResult<CertificateResponse> {
        let tenant_id = context.tenant_id;
        if tenant_id <= 0 {
            return Err(WebServiceError::Forbidden);
        }

        let (certificate_id, hostname) = self
            .repository
            .insert_certificate_pending(
                tenant_id,
                &request.domain_id,
                request.cert_type,
                request.auto_renew,
            )
            .await?;

        let cert_name = certificate_id.clone();

        let issue_result = self
            .certificate_issuer
            .issue(request.cert_type, &hostname, &cert_name)
            .await;

        let material = match issue_result {
            Ok(material) => material,
            Err(error) => {
                let _ = self
                    .repository
                    .fail_certificate(tenant_id, &certificate_id, &error.to_string())
                    .await;
                return Err(WebServiceError::Internal(error.to_string()));
            }
        };

        self.persist_issued_certificate(
            tenant_id,
            &certificate_id,
            request.auto_renew,
            material,
            "certificates.issue",
        )
        .await
    }
}
