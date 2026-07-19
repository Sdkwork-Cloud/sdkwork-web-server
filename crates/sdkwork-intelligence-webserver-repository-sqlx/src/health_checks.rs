use sdkwork_webserver_contract::{
    CreateHealthCheckRequest, HealthCheckPage, HealthCheckResponse, WebServiceError,
    WebServiceResult,
};
use sqlx::{any::AnyRow, Row};

use crate::support::{
    instant_write_expression, new_uuid, next_id, now_rfc3339, resolve_site_internal_id, store_error,
};
use crate::WebRepository;

impl WebRepository {
    pub(super) async fn list_health_checks_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
    ) -> WebServiceResult<HealthCheckPage> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;

        let count_row = sqlx::query(
            "SELECT COUNT(*) AS total FROM web_health_check
             WHERE tenant_id = $1 AND site_id = $2",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| store_error("count web_health_check", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let rows = sqlx::query(
            "SELECT uuid, check_type, check_url, status
             FROM web_health_check
             WHERE tenant_id = $1 AND site_id = $2
             ORDER BY created_at DESC",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("list web_health_check", error))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_health_check_row(row).map_err(|error| {
                WebServiceError::Internal(format!("map web_health_check row: {error}"))
            })?);
        }

        Ok(HealthCheckPage { items, total })
    }

    pub(super) async fn create_health_check_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateHealthCheckRequest,
    ) -> WebServiceResult<HealthCheckResponse> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();
        let engine = self.database_engine().await?;
        let now_expression = instant_write_expression(engine, "$7");
        let insert_sql = format!(
            "INSERT INTO web_health_check (
                id, uuid, tenant_id, site_id, check_type, check_url, status,
                created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, 1, {now_expression}, {now_expression}, 0
             )"
        );

        sqlx::query(&insert_sql)
            .bind(id)
            .bind(&uuid)
            .bind(tenant_id)
            .bind(site_internal_id)
            .bind(request.check_type)
            .bind(&request.url)
            .bind(&now)
            .execute(&self.pool)
            .await
            .map_err(|error| store_error("insert web_health_check", error))?;

        Ok(HealthCheckResponse {
            id: uuid,
            check_type: request.check_type,
            url: request.url.clone(),
            status: 1,
        })
    }
}

fn map_health_check_row(row: &AnyRow) -> Result<HealthCheckResponse, sqlx::Error> {
    Ok(HealthCheckResponse {
        id: row.try_get("uuid")?,
        check_type: row.try_get("check_type")?,
        url: row.try_get("check_url")?,
        status: row.try_get("status")?,
    })
}
