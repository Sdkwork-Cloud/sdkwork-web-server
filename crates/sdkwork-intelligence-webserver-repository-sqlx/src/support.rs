use sdkwork_database_config::DatabaseEngine;
use sdkwork_database_id::{uuid_v4, uuid_v4_with_prefix, SnowflakeIdGenerator};
use sdkwork_utils_rust::{crypto::sha256_hash, number::clamp};
use sdkwork_webserver_contract::WebServiceError;
use super::{EnginePool, EngineRow};
use sqlx::{Error as SqlxError, Row};

pub(crate) fn now_rfc3339() -> String {
    sdkwork_utils_rust::datetime::format_datetime(sdkwork_utils_rust::datetime::now(), None)
}

pub(crate) fn store_error(context: &str, error: SqlxError) -> WebServiceError {
    tracing::error!("{context}: {error}");
    match error {
        SqlxError::Database(db) if db.is_unique_violation() => {
            WebServiceError::conflict(db.message())
        }
        SqlxError::RowNotFound => WebServiceError::not_found("resource not found"),
        _ => WebServiceError::Internal(format!("{context}: {error}")),
    }
}

pub(crate) fn is_unique_violation(error: &SqlxError) -> bool {
    matches!(error, SqlxError::Database(database) if database.is_unique_violation())
}

pub(crate) fn pagination(page: i32, page_size: i32) -> (i32, i32, i64) {
    let page = page.max(1);
    let page_size = clamp(page_size, 1, 100);
    let offset = (i64::from(page) - 1) * i64::from(page_size);
    (page, page_size, offset)
}

pub(crate) fn next_id(generator: &SnowflakeIdGenerator) -> Result<i64, WebServiceError> {
    generator
        .generate()
        .map_err(|error| WebServiceError::Internal(error.to_string()))
}

pub(crate) fn new_uuid() -> String {
    uuid_v4()
}

pub(crate) fn new_agent_token() -> String {
    uuid_v4_with_prefix("wagent_")
}

pub(crate) fn sha256_hex(content: &str) -> String {
    sha256_hash(content.as_bytes())
}

pub(crate) fn bool_from_row(row: &EngineRow, column: &str) -> Result<bool, SqlxError> {
    if let Ok(value) = row.try_get::<bool, _>(column) {
        return Ok(value);
    }
    let value: i64 = row.try_get(column)?;
    Ok(value != 0)
}

pub(crate) fn json_from_row(
    row: &EngineRow,
    column: &str,
) -> Result<Option<serde_json::Value>, SqlxError> {
    let raw: Option<String> = row.try_get(column)?;
    Ok(raw.and_then(|text| serde_json::from_str(&text).ok()))
}

pub(crate) fn json_write_expression(engine: DatabaseEngine, placeholder: &str) -> String {
    match engine {
        DatabaseEngine::Sqlite => placeholder.to_string(),
        DatabaseEngine::Postgres => format!("CAST({placeholder} AS JSONB)"),
    }
}

pub(crate) fn instant_write_expression(engine: DatabaseEngine, placeholder: &str) -> String {
    match engine {
        DatabaseEngine::Sqlite => placeholder.to_string(),
        DatabaseEngine::Postgres => format!("CAST({placeholder} AS TIMESTAMPTZ)"),
    }
}

pub(crate) async fn resolve_site_internal_id(
    pool: &EnginePool,
    tenant_id: i64,
    site_uuid: &str,
) -> Result<i64, WebServiceError> {
    let row = sqlx::query(
        "SELECT id FROM web_site
         WHERE tenant_id = $1 AND uuid = $2 AND deleted_at IS NULL",
    )
    .bind(tenant_id)
    .bind(site_uuid)
    .fetch_optional(pool)
    .await
    .map_err(|error| store_error("resolve web_site id", error))?;

    row.and_then(|row| row.try_get::<i64, _>("id").ok())
        .ok_or_else(|| WebServiceError::not_found("site not found"))
}

pub(crate) async fn resolve_site_uuid(
    pool: &EnginePool,
    tenant_id: i64,
    site_internal_id: i64,
) -> Result<String, WebServiceError> {
    let row = sqlx::query(
        "SELECT uuid FROM web_site
         WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL",
    )
    .bind(tenant_id)
    .bind(site_internal_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| store_error("resolve web_site uuid", error))?;

    row.and_then(|row| row.try_get::<String, _>("uuid").ok())
        .ok_or_else(|| WebServiceError::not_found("site not found"))
}

#[cfg(test)]
mod tests {
    use super::pagination;

    #[test]
    fn pagination_clamps_inputs_and_computes_offset_without_i32_overflow() {
        assert_eq!(pagination(-10, -20), (1, 1, 0));
        assert_eq!(pagination(2, 500), (2, 100, 100));
        assert_eq!(
            pagination(i32::MAX, i32::MAX),
            (i32::MAX, 100, 214_748_364_600)
        );
    }
}
