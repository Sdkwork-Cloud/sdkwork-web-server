use sdkwork_utils_rust::slugify;
use sdkwork_webserver_contract::{
    CreateSiteRequest, ListSitesQuery, SitePage, SiteResponse, UpdateSiteRequest, WebServiceError,
    WebServiceResult,
};
use super::{EngineRow, WebRepository};
use sqlx::Row;

use super::support::{
    instant_write_expression, json_from_row, json_write_expression, new_uuid, next_id, now_rfc3339,
    pagination, store_error,
};

impl WebRepository {
    pub(super) async fn list_sites_repo(
        &self,
        tenant_id: i64,
        query: &ListSitesQuery,
    ) -> WebServiceResult<SitePage> {
        let (page, page_size, offset) = pagination(query.page, query.page_size);
        let keyword = query
            .keyword
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| format!("%{value}%"));
        let count_sql = "SELECT COUNT(*) AS total FROM web_site
             WHERE tenant_id = $1 AND deleted_at IS NULL
               AND ($2 IS NULL OR status = $2)
               AND ($3 IS NULL OR site_type = $3)
               AND ($4 IS NULL OR name LIKE $4 OR slug LIKE $4)";
        let list_sql = "SELECT uuid, name, slug, description, site_type, status,
                    CAST(runtime_config AS TEXT) AS runtime_config,
                    CAST(created_at AS TEXT) AS created_at,
                    CAST(updated_at AS TEXT) AS updated_at
             FROM web_site
             WHERE tenant_id = $1 AND deleted_at IS NULL
               AND ($2 IS NULL OR status = $2)
               AND ($3 IS NULL OR site_type = $3)
               AND ($4 IS NULL OR name LIKE $4 OR slug LIKE $4)
             ORDER BY updated_at DESC, id DESC LIMIT $5 OFFSET $6";

        let count_query = sqlx::query(count_sql)
            .bind(tenant_id)
            .bind(query.status)
            .bind(query.site_type)
            .bind(keyword.as_deref());
        let list_query = sqlx::query(list_sql)
            .bind(tenant_id)
            .bind(query.status)
            .bind(query.site_type)
            .bind(keyword.as_deref())
            .bind(page_size)
            .bind(offset);

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
        let org_id = organization_id.unwrap_or(0);
        let engine = self.database_engine().await?;
        let runtime_config_expression = json_write_expression(engine, "$10");
        let now_expression = instant_write_expression(engine, "$11");
        let insert_sql = format!(
            "INSERT INTO web_site (
                id, uuid, tenant_id, organization_id, user_id, name, slug, description,
                site_type, status, runtime_config, metadata, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, 0,
                {runtime_config_expression}, '{{}}', {now_expression}, {now_expression}, 0
             )"
        );

        sqlx::query(&insert_sql)
            .bind(id)
            .bind(&uuid)
            .bind(tenant_id)
            .bind(org_id)
            .bind(actor_id)
            .bind(&request.name)
            .bind(&slug)
            .bind(&request.description)
            .bind(request.site_type)
            .bind(runtime_config.to_string())
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
            "SELECT uuid, name, slug, description, site_type, status,
                    CAST(runtime_config AS TEXT) AS runtime_config,
                    CAST(created_at AS TEXT) AS created_at,
                    CAST(updated_at AS TEXT) AS updated_at
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
        let now = now_rfc3339();
        let engine = self.database_engine().await?;
        let runtime_config_expression = json_write_expression(engine, "$5");
        let now_expression = instant_write_expression(engine, "$6");
        let update_sql = format!(
            "UPDATE web_site
             SET name = $3, description = $4, runtime_config = {runtime_config_expression},
                 updated_at = {now_expression}, version = version + 1
             WHERE tenant_id = $1 AND uuid = $2 AND deleted_at IS NULL"
        );

        let updated = sqlx::query(&update_sql)
            .bind(tenant_id)
            .bind(site_id)
            .bind(name)
            .bind(description)
            .bind(runtime_config.to_string())
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
        let engine = self.database_engine().await?;
        let now_expression = instant_write_expression(engine, "$3");
        let update_sql = format!(
            "UPDATE web_site
             SET deleted_at = {now_expression}, deleted_by = $4,
                 updated_at = {now_expression}, version = version + 1
             WHERE tenant_id = $1 AND uuid = $2 AND deleted_at IS NULL"
        );
        let result = sqlx::query(&update_sql)
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
        let engine = self.database_engine().await?;
        let now_expression = instant_write_expression(engine, "$4");
        let update_sql = format!(
            "UPDATE web_site
             SET status = $3, updated_at = {now_expression}, version = version + 1
             WHERE tenant_id = $1 AND uuid = $2 AND deleted_at IS NULL"
        );
        let result = sqlx::query(&update_sql)
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

fn map_site_row(row: &EngineRow) -> Result<SiteResponse, sqlx::Error> {
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
