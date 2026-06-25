use chrono::{DateTime, Duration, Utc};
use sdkwork_webserver_contract::{
    CertificateIssueUpdate, CertificatePage, CertificateResponse, CreateCertificateRequest,
    WebServiceError, WebServiceResult,
};
use serde_json::json;
use sqlx::{any::AnyRow, Row};

use crate::domains_lookup::{cert_name_from_hostname, resolve_domain_by_uuid};
use crate::support::{new_uuid, next_id, now_rfc3339, pagination, store_error};
use crate::WebRepository;

impl WebRepository {
    pub(super) async fn list_certificates_repo(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<CertificatePage> {
        let (_page, page_size, offset) = pagination(page, page_size);

        let count_row =
            sqlx::query("SELECT COUNT(*) AS total FROM web_certificate WHERE tenant_id = $1")
                .bind(tenant_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|error| store_error("count web_certificate", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let rows = sqlx::query(
            "SELECT uuid, cert_name, cert_type, issuer, not_before, not_after, auto_renew, status, created_at
             FROM web_certificate
             WHERE tenant_id = $1
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(tenant_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("list web_certificate", error))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_certificate_row(row).map_err(|error| {
                WebServiceError::Internal(format!("map web_certificate row: {error}"))
            })?);
        }

        Ok(CertificatePage { items, total })
    }

    pub(super) async fn insert_certificate_pending_repo(
        &self,
        tenant_id: i64,
        domain_uuid: &str,
        cert_type: i32,
        auto_renew: bool,
    ) -> WebServiceResult<(String, String)> {
        let domain = resolve_domain_by_uuid(&self.pool, tenant_id, domain_uuid).await?;
        if cert_type == 1 && !domain.is_verified {
            return Err(WebServiceError::validation(
                "domain must be verified before Let's Encrypt issuance",
            ));
        }

        let cert_name = cert_name_from_hostname(&domain.hostname);
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();

        sqlx::query(
            "INSERT INTO web_certificate (
                id, uuid, tenant_id, site_id, domain_id, cert_name, cert_type,
                auto_renew, renewal_status, status, metadata, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, 2, 0, '{}', $9, $9, 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(domain.site_internal_id)
        .bind(domain.internal_id)
        .bind(&cert_name)
        .bind(cert_type)
        .bind(auto_renew)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert web_certificate pending", error))?;

        Ok((uuid, domain.hostname))
    }

    pub(super) async fn list_certificates_due_for_renewal_repo(
        &self,
        renew_before_days: u32,
        limit: i32,
    ) -> WebServiceResult<Vec<sdkwork_webserver_contract::CertificateRenewalCandidate>> {
        use sdkwork_webserver_contract::CertificateRenewalCandidate;

        let rows = sqlx::query(
            "SELECT c.tenant_id, c.uuid, c.cert_type, c.cert_name, c.auto_renew, c.not_after,
                    COALESCE(d.hostname, c.subject, c.cert_name) AS hostname
             FROM web_certificate c
             LEFT JOIN web_domain d ON d.id = c.domain_id
             WHERE c.auto_renew = $1
               AND c.status = 1
               AND c.renewal_status IN (0, 3)
               AND c.cert_type IN (1, 3)
               AND c.not_after IS NOT NULL
             ORDER BY c.not_after ASC
             LIMIT $2",
        )
        .bind(true)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("list web_certificate renewal candidates", error))?;

        let mut items = Vec::new();
        for row in &rows {
            let not_after: String = row.try_get("not_after").map_err(|error| {
                WebServiceError::Internal(format!("renewal candidate not_after: {error}"))
            })?;
            if !certificate_due_for_renewal(&not_after, renew_before_days) {
                continue;
            }
            items.push(CertificateRenewalCandidate {
                tenant_id: row.try_get("tenant_id").map_err(|error| {
                    WebServiceError::Internal(format!("renewal candidate tenant_id: {error}"))
                })?,
                certificate_id: row.try_get("uuid").map_err(|error| {
                    WebServiceError::Internal(format!("renewal candidate uuid: {error}"))
                })?,
                cert_type: row.try_get("cert_type").map_err(|error| {
                    WebServiceError::Internal(format!("renewal candidate cert_type: {error}"))
                })?,
                cert_name: row.try_get("cert_name").map_err(|error| {
                    WebServiceError::Internal(format!("renewal candidate cert_name: {error}"))
                })?,
                hostname: row.try_get("hostname").map_err(|error| {
                    WebServiceError::Internal(format!("renewal candidate hostname: {error}"))
                })?,
                auto_renew: row.try_get("auto_renew").map_err(|error| {
                    WebServiceError::Internal(format!("renewal candidate auto_renew: {error}"))
                })?,
                not_after,
            });
        }
        Ok(items)
    }

    pub(super) async fn mark_certificate_renewing_repo(
        &self,
        tenant_id: i64,
        certificate_uuid: &str,
    ) -> WebServiceResult<bool> {
        let now = now_rfc3339();
        let result = sqlx::query(
            "UPDATE web_certificate
             SET renewal_status = 1, updated_at = $3, version = version + 1
             WHERE tenant_id = $1 AND uuid = $2 AND status = 1 AND renewal_status IN (0, 3)",
        )
        .bind(tenant_id)
        .bind(certificate_uuid)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("mark web_certificate renewing", error))?;
        Ok(result.rows_affected() > 0)
    }

    pub(super) async fn fail_certificate_renewal_repo(
        &self,
        tenant_id: i64,
        certificate_uuid: &str,
        reason: &str,
    ) -> WebServiceResult<()> {
        let row =
            sqlx::query("SELECT metadata FROM web_certificate WHERE tenant_id = $1 AND uuid = $2")
                .bind(tenant_id)
                .bind(certificate_uuid)
                .fetch_optional(&self.pool)
                .await
                .map_err(|error| {
                    store_error("load web_certificate metadata for renewal failure", error)
                })?
                .ok_or_else(|| WebServiceError::not_found("certificate not found"))?;

        let existing_raw: String = row.try_get("metadata").map_err(|error| {
            WebServiceError::Internal(format!("renewal failure metadata: {error}"))
        })?;
        let mut existing: serde_json::Value =
            serde_json::from_str(&existing_raw).unwrap_or_else(|_| json!({}));
        if let Some(object) = existing.as_object_mut() {
            object.insert(
                "renewalFailureReason".to_string(),
                serde_json::Value::String(reason.to_string()),
            );
        }

        let now = now_rfc3339();
        sqlx::query(
            "UPDATE web_certificate
             SET renewal_status = 3, metadata = $3, updated_at = $4, version = version + 1
             WHERE tenant_id = $1 AND uuid = $2",
        )
        .bind(tenant_id)
        .bind(certificate_uuid)
        .bind(existing.to_string())
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("fail web_certificate renewal", error))?;
        Ok(())
    }

    pub(super) async fn finalize_certificate_repo(
        &self,
        tenant_id: i64,
        certificate_uuid: &str,
        update: &CertificateIssueUpdate,
    ) -> WebServiceResult<CertificateResponse> {
        let metadata = json!({
            "encryptedPrivateKey": update.encrypted_private_key,
            "certPem": update.cert_pem,
            "chainPem": update.chain_pem,
            "keyVersion": 1
        });
        let now = now_rfc3339();

        let result = sqlx::query(
            "UPDATE web_certificate SET
                cert_name = $3,
                cert_type = $4,
                issuer = $5,
                subject = $6,
                san_list = $7,
                fingerprint = $8,
                cert_path = $9,
                key_path = $10,
                chain_path = $11,
                not_before = $12,
                not_after = $13,
                auto_renew = $14,
                renewal_status = 0,
                status = 1,
                metadata = $15,
                updated_at = $16,
                version = version + 1
             WHERE tenant_id = $1 AND uuid = $2",
        )
        .bind(tenant_id)
        .bind(certificate_uuid)
        .bind(&update.cert_name)
        .bind(update.cert_type)
        .bind(&update.issuer)
        .bind(&update.subject)
        .bind(&update.san_list)
        .bind(&update.fingerprint)
        .bind(&update.cert_path)
        .bind(&update.key_path)
        .bind(update.chain_path.as_deref())
        .bind(&update.not_before)
        .bind(&update.not_after)
        .bind(update.auto_renew)
        .bind(metadata.to_string())
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("finalize web_certificate", error))?;

        if result.rows_affected() == 0 {
            return Err(WebServiceError::not_found("certificate not found"));
        }

        self.retrieve_certificate_repo(tenant_id, certificate_uuid)
            .await
    }

    pub(super) async fn fail_certificate_repo(
        &self,
        tenant_id: i64,
        certificate_uuid: &str,
        reason: &str,
    ) -> WebServiceResult<()> {
        let metadata = json!({ "failureReason": reason });
        let now = now_rfc3339();
        sqlx::query(
            "UPDATE web_certificate SET renewal_status = 3, status = 0, metadata = $3, updated_at = $4, version = version + 1
             WHERE tenant_id = $1 AND uuid = $2",
        )
        .bind(tenant_id)
        .bind(certificate_uuid)
        .bind(metadata.to_string())
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("fail web_certificate", error))?;
        Ok(())
    }

    pub(super) async fn retrieve_certificate_repo(
        &self,
        tenant_id: i64,
        certificate_uuid: &str,
    ) -> WebServiceResult<CertificateResponse> {
        let row = sqlx::query(
            "SELECT uuid, cert_name, cert_type, issuer, not_before, not_after, auto_renew, status, created_at
             FROM web_certificate WHERE tenant_id = $1 AND uuid = $2",
        )
        .bind(tenant_id)
        .bind(certificate_uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("retrieve web_certificate", error))?
        .ok_or_else(|| WebServiceError::not_found("certificate not found"))?;

        map_certificate_row(&row)
            .map_err(|error| WebServiceError::Internal(format!("map web_certificate row: {error}")))
    }

    pub(super) async fn create_certificate_repo(
        &self,
        tenant_id: i64,
        request: &CreateCertificateRequest,
    ) -> WebServiceResult<CertificateResponse> {
        let (uuid, _) = self
            .insert_certificate_pending_repo(
                tenant_id,
                &request.domain_id,
                request.cert_type,
                request.auto_renew,
            )
            .await?;
        self.retrieve_certificate_repo(tenant_id, &uuid).await
    }
}

fn map_certificate_row(row: &AnyRow) -> Result<CertificateResponse, sqlx::Error> {
    Ok(CertificateResponse {
        id: row.try_get("uuid")?,
        cert_name: row.try_get("cert_name")?,
        cert_type: row.try_get("cert_type").ok(),
        issuer: row.try_get("issuer").ok(),
        not_before: row.try_get("not_before").ok(),
        not_after: row.try_get("not_after").ok(),
        auto_renew: row.try_get("auto_renew").ok(),
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
    })
}

fn certificate_due_for_renewal(not_after: &str, renew_before_days: u32) -> bool {
    let Ok(not_after) = DateTime::parse_from_rfc3339(not_after) else {
        return false;
    };
    let threshold = Utc::now() + Duration::days(i64::from(renew_before_days));
    not_after.with_timezone(&Utc) <= threshold
}
