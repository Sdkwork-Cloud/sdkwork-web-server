use sdkwork_webserver_contract::WebServiceError;
use sqlx::Row;

use super::EnginePool;

#[derive(Clone, Debug)]
pub struct DomainRecord {
    pub internal_id: i64,
    pub site_internal_id: i64,
    pub hostname: String,
    pub is_verified: bool,
}

pub(crate) async fn resolve_domain_by_uuid(
    pool: &EnginePool,
    tenant_id: i64,
    domain_uuid: &str,
) -> Result<DomainRecord, WebServiceError> {
    let row = sqlx::query(
        "SELECT d.id, d.site_id, d.hostname, d.is_verified
         FROM web_domain d
         INNER JOIN web_site s ON s.id = d.site_id
         WHERE d.tenant_id = $1 AND d.uuid = $2 AND d.deleted_at IS NULL AND s.deleted_at IS NULL",
    )
    .bind(tenant_id)
    .bind(domain_uuid)
    .fetch_optional(pool)
    .await
    .map_err(|error| super::support::store_error("resolve web_domain", error))?;

    let row = row.ok_or_else(|| WebServiceError::not_found("domain not found"))?;
    Ok(DomainRecord {
        internal_id: row
            .try_get("id")
            .map_err(|error| WebServiceError::Internal(format!("resolve domain id: {error}")))?,
        site_internal_id: row.try_get("site_id").map_err(|error| {
            WebServiceError::Internal(format!("resolve domain site_id: {error}"))
        })?,
        hostname: row.try_get("hostname").map_err(|error| {
            WebServiceError::Internal(format!("resolve domain hostname: {error}"))
        })?,
        is_verified: super::support::bool_from_row(&row, "is_verified").map_err(|error| {
            WebServiceError::Internal(format!("resolve domain is_verified: {error}"))
        })?,
    })
}

pub(crate) fn cert_name_from_hostname(hostname: &str) -> String {
    hostname
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
