use sdkwork_utils_rust::slugify;
use sdkwork_webserver_contract::{
    CreateSiteRequest, ListSitesQuery, SitePage, SiteResponse, UpdateSiteRequest, WebServiceError,
    WebServiceResult,
};
use sqlx::{any::AnyRow, Row};

use crate::support::{json_from_row, new_uuid, next_id, now_rfc3339, pagination, store_error};
use crate::WebRepository;

impl WebRepository {
    pub(super) async fn list_sites_repo(
        &self,
        tenant_id: i64,
        query: &ListSitesQuery,
    ) -> WebServiceResult<SitePage> {
        let (page, page_size, offset) = pagination(query.page, query.page_size);
        let mut count_sql = String::from(
            "SELECT COUNT(*) AS total FROM web_site
             WHERE tenant_id = $1 AND deleted_at IS NULL",
        );
        let mut list_sql = String::from(
            "SELECT uuid, name, slug, description, site_type, status, runtime_config, created_at, updated_at
             FROM web_site
             WHERE tenant_id = $1 AND deleted_at IS NULL",
        );

        let mut bind_index = 2u8;
        let mut extra_binds: Vec<String> = Vec::new();

        if let Some(status) = query.status {
            let clause = format!(" AND status = ${bind_index}");
            count_sql.push_str(&clause);
            list_sql.push_str(&clause);
            extra_binds.push(status.to_string());
            bind_index += 1;
        }
        if let Some(site_type) = query.site_type {
            let clause = format!(" AND site_type = ${bind_index}");
            count_sql.push_str(&clause);
            list_sql.push_str(&clause);
            extra_binds.push(site_type.to_string());
            bind_index += 1;
        }
        if let Some(keyword) = query
            .keyword
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            let clause = format!(
                " AND (name LIKE ${bind_index} OR slug LIKE ${})",
                bind_index + 1
            );
            count_sql.push_str(&clause);
            list_sql.push_str(&clause);
            let pattern = format!("%{}%", keyword.trim());
            extra_binds.push(pattern.clone());
            extra_binds.push(pattern);
        }

        list_sql.push_str(&format!(
            " ORDER BY updated_at DESC LIMIT ${bind_index} OFFSET ${}",
            bind_index + 1
        ));

        let mut count_query = sqlx::query(&count_sql).bind(tenant_id);
        let mut list_query = sqlx::query(&list_sql).bind(tenant_id);
        for value in &extra_binds {
            count_query = count_query.bind(value);
            list_query = list_query.bind(value);
        }
        list_query = list_query.bind(page_size).bind(offset);

        let count_row = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|error| store_error("count web_site", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let rows = list_query
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("list web_site", error))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_site_row(row).map_err(|error| {
                WebServiceError::Internal(format!("map web_site row: {error}"))
            })?);
        }

        Ok(SitePage {
            items,
            total,
            page,
            page_size,
        })
    }

    pub(super) async fn create_site_repo(
        &self,
        tenant_id: i64,
        organization_id: Option<i64>,
        actor_id: Option<i64>,
        request: &CreateSiteRequest,
    ) -> WebServiceResult<SiteResponse> {
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let slug = request
            .slug
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(slugify)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| slugify(&request.name));
        if slug.is_empty() {
            return Err(WebServiceError::validation("slug cannot be empty"));
        }
        let now = now_rfc3339();
        let runtime_config = request
            .runtime_config
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));
        let runtime_config_text = runtime_config.to_string();
        let org_id = organization_id.unwrap_or(0);

        sqlx::query(
            "INSERT INTO web_site (
                id, uuid, tenant_id, organization_id, user_id, name, slug, description,
                site_type, status, runtime_config, metadata, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, 0, $10, '{}', $11, $11, 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(org_id)
        .bind(actor_id)
        .bind(&request.name)
        .bind(&slug)
        .bind(&request.description)
        .bind(request.site_type)
        .bind(&runtime_config_text)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert web_site", error))?;

        self.retrieve_site_repo(tenant_id, &uuid).await
    }

    pub(super) async fn retrieve_site_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
    ) -> WebServiceResult<SiteResponse> {
        let row = sqlx::query(
            "SELECT uuid, name, slug, description, site_type, status, runtime_config, created_at, updated_at
             FROM web_site
             WHERE tenant_id = $1 AND uuid = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(site_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("retrieve web_site", error))?
        .ok_or_else(|| WebServiceError::not_found("site not found"))?;

        map_site_row(&row).map_err(|error| WebServiceError::Internal(error.to_string()))
    }

    pub(super) async fn update_site_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &UpdateSiteRequest,
    ) -> WebServiceResult<SiteResponse> {
        let existing = self.retrieve_site_repo(tenant_id, site_id).await?;
        let name = request.name.as_ref().unwrap_or(&existing.name);
        let description = request
            .description
            .as_ref()
            .or(existing.description.as_ref());
        let runtime_config = request
            .runtime_config
            .clone()
            .or(existing.runtime_config)
            .unwrap_or_else(|| serde_json::json!({}));
        let runtime_config_text = runtime_config.to_string();
        let now = now_rfc3339();

        let updated = sqlx::query(
            "UPDATE web_site
             SET name = $3, description = $4, runtime_config = $5, updated_at = $6, version = version + 1
             WHERE tenant_id = $1 AND uuid = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(site_id)
        .bind(name)
        .bind(description)
        .bind(&runtime_config_text)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("update web_site", error))?;

        if updated.rows_affected() == 0 {
            return Err(WebServiceError::not_found("site not found"));
        }

        self.retrieve_site_repo(tenant_id, site_id).await
    }

    pub(super) async fn delete_site_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        actor_id: Option<i64>,
    ) -> WebServiceResult<()> {
        let now = now_rfc3339();
        let result = sqlx::query(
            "UPDATE web_site
             SET deleted_at = $3, deleted_by = $4, updated_at = $3, version = version + 1
             WHERE tenant_id = $1 AND uuid = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(site_id)
        .bind(&now)
        .bind(actor_id)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("delete web_site", error))?;

        if result.rows_affected() == 0 {
            return Err(WebServiceError::not_found("site not found"));
        }
        Ok(())
    }

    pub(super) async fn set_site_status_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        status: i32,
    ) -> WebServiceResult<SiteResponse> {
        let now = now_rfc3339();
        let result = sqlx::query(
            "UPDATE web_site
             SET status = $3, updated_at = $4, version = version + 1
             WHERE tenant_id = $1 AND uuid = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(site_id)
        .bind(status)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("update web_site status", error))?;

        if result.rows_affected() == 0 {
            return Err(WebServiceError::not_found("site not found"));
        }

        self.retrieve_site_repo(tenant_id, site_id).await
    }
}

fn map_site_row(row: &AnyRow) -> Result<SiteResponse, sqlx::Error> {
    Ok(SiteResponse {
        id: row.try_get("uuid")?,
        name: row.try_get("name")?,
        slug: row.try_get("slug")?,
        description: row.try_get("description").ok(),
        site_type: row.try_get("site_type")?,
        status: row.try_get("status")?,
        runtime_config: json_from_row(row, "runtime_config")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
