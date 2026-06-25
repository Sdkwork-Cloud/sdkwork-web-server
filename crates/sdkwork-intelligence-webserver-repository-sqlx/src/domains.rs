use sdkwork_webserver_contract::{
    CreateDomainRequest, DomainPage, DomainResponse, DomainVerifyResponse, WebServiceError,
    WebServiceResult,
};
use sqlx::{any::AnyRow, Row};

use crate::support::{
    bool_from_row, new_uuid, next_id, now_rfc3339, pagination, resolve_site_internal_id,
    store_error,
};
use crate::WebRepository;

impl WebRepository {
    pub(super) async fn list_domains_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<DomainPage> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let (_page, page_size, offset) = pagination(page, page_size);

        let count_row = sqlx::query(
            "SELECT COUNT(*) AS total FROM web_domain
             WHERE tenant_id = $1 AND site_id = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| store_error("count web_domain", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let rows = sqlx::query(
            "SELECT uuid, hostname, is_primary, is_verified, ssl_enabled, ssl_provider, status, created_at
             FROM web_domain
             WHERE tenant_id = $1 AND site_id = $2 AND deleted_at IS NULL
             ORDER BY created_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("list web_domain", error))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_domain_row(row).map_err(|error| {
                WebServiceError::Internal(format!("map web_domain row: {error}"))
            })?);
        }

        Ok(DomainPage { items, total })
    }

    pub(super) async fn create_domain_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateDomainRequest,
    ) -> WebServiceResult<DomainResponse> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();
        let verify_token = new_uuid();

        if request.is_primary {
            sqlx::query(
                "UPDATE web_domain SET is_primary = 0, updated_at = $3
                 WHERE tenant_id = $1 AND site_id = $2 AND deleted_at IS NULL",
            )
            .bind(tenant_id)
            .bind(site_internal_id)
            .bind(&now)
            .execute(&self.pool)
            .await
            .map_err(|error| store_error("clear primary web_domain", error))?;
        }

        sqlx::query(
            "INSERT INTO web_domain (
                id, uuid, tenant_id, site_id, hostname, is_primary, is_verified, verify_token,
                ssl_enabled, ssl_provider, status, metadata, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, 0, $7, $8, $9, 0, '{}', $10, $10, 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(&request.hostname)
        .bind(request.is_primary)
        .bind(&verify_token)
        .bind(request.ssl_enabled)
        .bind(&request.ssl_provider)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert web_domain", error))?;

        self.retrieve_domain_repo(tenant_id, site_id, &uuid).await
    }

    pub(super) async fn retrieve_domain_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> WebServiceResult<DomainResponse> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let row = sqlx::query(
            "SELECT uuid, hostname, is_primary, is_verified, ssl_enabled, ssl_provider, status, created_at
             FROM web_domain
             WHERE tenant_id = $1 AND site_id = $2 AND uuid = $3 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(domain_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("retrieve web_domain", error))?
        .ok_or_else(|| WebServiceError::not_found("domain not found"))?;

        map_domain_row(&row).map_err(|error| WebServiceError::Internal(error.to_string()))
    }

    pub(super) async fn delete_domain_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> WebServiceResult<()> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let now = now_rfc3339();
        let result = sqlx::query(
            "UPDATE web_domain
             SET deleted_at = $4, updated_at = $4, version = version + 1
             WHERE tenant_id = $1 AND site_id = $2 AND uuid = $3 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(domain_id)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("delete web_domain", error))?;

        if result.rows_affected() == 0 {
            return Err(WebServiceError::not_found("domain not found"));
        }
        Ok(())
    }

    pub(super) async fn verify_domain_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> WebServiceResult<DomainVerifyResponse> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let row = sqlx::query(
            "SELECT is_verified, verify_token FROM web_domain
             WHERE tenant_id = $1 AND site_id = $2 AND uuid = $3 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(domain_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("verify web_domain lookup", error))?
        .ok_or_else(|| WebServiceError::not_found("domain not found"))?;

        let is_verified = bool_from_row(&row, "is_verified").unwrap_or(false);
        let verify_token: Option<String> = row.try_get("verify_token").ok();

        if is_verified {
            return Ok(DomainVerifyResponse {
                verified: true,
                verify_token: None,
            });
        }

        let now = now_rfc3339();
        sqlx::query(
            "UPDATE web_domain
             SET is_verified = 1, status = 1, updated_at = $4, version = version + 1
             WHERE tenant_id = $1 AND site_id = $2 AND uuid = $3 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(domain_id)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("verify web_domain", error))?;

        Ok(DomainVerifyResponse {
            verified: true,
            verify_token,
        })
    }
}

fn map_domain_row(row: &AnyRow) -> Result<DomainResponse, sqlx::Error> {
    Ok(DomainResponse {
        id: row.try_get("uuid")?,
        hostname: row.try_get("hostname")?,
        is_primary: bool_from_row(row, "is_primary")?,
        is_verified: bool_from_row(row, "is_verified")?,
        ssl_enabled: bool_from_row(row, "ssl_enabled")?,
        ssl_provider: row.try_get("ssl_provider").ok(),
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
    })
}
