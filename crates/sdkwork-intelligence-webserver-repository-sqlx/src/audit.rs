use sdkwork_intelligence_webserver_service::AuditLogWrite;
use sdkwork_webserver_contract::{
    AuditLogPage, AuditLogResponse, WebServiceError, WebServiceResult,
};
use super::{EngineRow, WebRepository};
use sqlx::Row;

use super::support::{
    instant_write_expression, new_uuid, next_id, now_rfc3339, pagination, store_error,
};

impl WebRepository {
    pub(super) async fn list_audit_logs_repo(
        &self,
        tenant_id: Option<i64>,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<AuditLogPage> {
        let (page, page_size, offset) = pagination(page, page_size);

        let (count_row, rows) = if let Some(tenant_id) = tenant_id {
            let count_row =
                sqlx::query("SELECT COUNT(*) AS total FROM web_audit_log WHERE tenant_id = $1")
                    .bind(tenant_id)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|error| store_error("count web_audit_log", error))?;

            let rows = sqlx::query(
                "SELECT uuid, action, target_type, CAST(created_at AS TEXT) AS created_at
                 FROM web_audit_log
                 WHERE tenant_id = $1
                 ORDER BY created_at DESC, id DESC LIMIT $2 OFFSET $3",
            )
            .bind(tenant_id)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("list web_audit_log", error))?;

            (count_row, rows)
        } else {
            let count_row = sqlx::query("SELECT COUNT(*) AS total FROM web_audit_log")
                .fetch_one(&self.pool)
                .await
                .map_err(|error| store_error("count web_audit_log", error))?;

            let rows = sqlx::query(
                "SELECT uuid, action, target_type, CAST(created_at AS TEXT) AS created_at
                 FROM web_audit_log
                 ORDER BY created_at DESC, id DESC LIMIT $1 OFFSET $2",
            )
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("list web_audit_log", error))?;

            (count_row, rows)
        };

        let total: i64 = count_row.try_get("total").unwrap_or(0);
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_audit_log_row(row).map_err(|error| {
                WebServiceError::Internal(format!("map web_audit_log row: {error}"))
            })?);
        }

        Ok(AuditLogPage {
            items,
            total,
            page,
            page_size,
        })
    }

    pub(super) async fn insert_audit_log_repo(
        &self,
        entry: AuditLogWrite<'_>,
    ) -> WebServiceResult<()> {
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();
        let engine = self.database_engine().await?;
        let now_expression = instant_write_expression(engine, "$10");
        let insert_sql = format!(
            "INSERT INTO web_audit_log (
                id, uuid, tenant_id, organization_id, operator_id, action, target_type,
                target_id, target_uuid, metadata, created_at
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, '{{}}', {now_expression}
             )"
        );

        sqlx::query(&insert_sql)
            .bind(id)
            .bind(&uuid)
            .bind(entry.tenant_id)
            .bind(entry.organization_id)
            .bind(entry.operator_id)
            .bind(entry.action)
            .bind(entry.target_type)
            .bind(entry.target_id)
            .bind(entry.target_uuid)
            .bind(&now)
            .execute(&self.pool)
            .await
            .map_err(|error| store_error("insert web_audit_log", error))?;

        Ok(())
    }
}

fn map_audit_log_row(row: &EngineRow) -> Result<AuditLogResponse, sqlx::Error> {
    Ok(AuditLogResponse {
        id: row.try_get("uuid")?,
        action: row.try_get("action")?,
        resource: row.try_get("target_type")?,
        created_at: row.try_get("created_at")?,
    })
}
