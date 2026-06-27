use sdkwork_webserver_contract::{
    CreateNginxConfigRequest, ListNginxConfigsQuery, NginxConfigPage, NginxConfigResponse,
    NginxReloadResponse, NginxStatusResponse, NginxValidateResponse, UpdateNginxConfigRequest,
    WebServiceError, WebServiceResult,
};
use sqlx::{any::AnyRow, Row};

use crate::support::{
    bool_from_row, new_uuid, next_id, now_rfc3339, pagination, resolve_site_internal_id,
    resolve_site_uuid, sha256_hex, store_error,
};
use crate::WebRepository;

impl WebRepository {
    pub(super) async fn list_nginx_configs_repo(
        &self,
        tenant_id: Option<i64>,
        query: &ListNginxConfigsQuery,
    ) -> WebServiceResult<NginxConfigPage> {
        let (page, page_size, offset) = pagination(query.page, query.page_size);
        let mut count_sql =
            String::from("SELECT COUNT(*) AS total FROM web_nginx_config WHERE 1=1");
        let mut list_sql = String::from(
            "SELECT uuid, site_id, config_name, config_type, is_active, status
             FROM web_nginx_config WHERE 1=1",
        );
        let mut binds: Vec<BindValue> = Vec::new();

        if let Some(tenant_id) = tenant_id {
            let index = binds.len() + 1;
            let clause = format!(" AND tenant_id = ${index}");
            count_sql.push_str(&clause);
            list_sql.push_str(&clause);
            binds.push(BindValue::I64(tenant_id));
        }
        if let Some(site_uuid) = query.site_id.as_deref() {
            if let Some(tenant_id) = tenant_id {
                let site_internal_id =
                    resolve_site_internal_id(&self.pool, tenant_id, site_uuid).await?;
                let index = binds.len() + 1;
                let clause = format!(" AND site_id = ${index}");
                count_sql.push_str(&clause);
                list_sql.push_str(&clause);
                binds.push(BindValue::I64(site_internal_id));
            }
        }
        if let Some(config_type) = query.config_type {
            let index = binds.len() + 1;
            let clause = format!(" AND config_type = ${index}");
            count_sql.push_str(&clause);
            list_sql.push_str(&clause);
            binds.push(BindValue::I32(config_type));
        }
        if let Some(is_active) = query.is_active {
            let index = binds.len() + 1;
            let clause = format!(" AND is_active = ${index}");
            count_sql.push_str(&clause);
            list_sql.push_str(&clause);
            binds.push(BindValue::Bool(is_active));
        }

        let limit_index = binds.len() + 1;
        let offset_index = binds.len() + 2;
        list_sql.push_str(&format!(
            " ORDER BY updated_at DESC LIMIT ${limit_index} OFFSET ${offset_index}"
        ));

        let count_row = apply_binds(sqlx::query(&count_sql), &binds)
            .fetch_one(&self.pool)
            .await
            .map_err(|error| store_error("count web_nginx_config", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let mut list_query = apply_binds(sqlx::query(&list_sql), &binds);
        list_query = list_query.bind(page_size).bind(offset);
        let rows = list_query
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("list web_nginx_config", error))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(
                map_nginx_config_row(&self.pool, tenant_id.unwrap_or(0), row)
                    .await
                    .map_err(|error| {
                        WebServiceError::Internal(format!("map web_nginx_config row: {error}"))
                    })?,
            );
        }

        Ok(NginxConfigPage {
            items,
            total,
            page,
            page_size,
        })
    }

    pub(super) async fn create_nginx_config_repo(
        &self,
        tenant_id: i64,
        request: &CreateNginxConfigRequest,
    ) -> WebServiceResult<NginxConfigResponse> {
        let site_internal_id =
            resolve_site_internal_id(&self.pool, tenant_id, &request.site_id).await?;
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();
        let config_hash = sha256_hex(&request.config_content);

        sqlx::query(
            "INSERT INTO web_nginx_config (
                id, uuid, tenant_id, site_id, config_type, config_name, config_content, config_hash,
                is_active, status, metadata, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, 0, 0, '{}', $9, $9, 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(request.config_type)
        .bind(&request.config_name)
        .bind(&request.config_content)
        .bind(&config_hash)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert web_nginx_config", error))?;

        self.retrieve_nginx_config_repo(Some(tenant_id), &uuid)
            .await
    }

    pub(super) async fn retrieve_nginx_config_repo(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> WebServiceResult<NginxConfigResponse> {
        let row = if let Some(tenant_id) = tenant_id {
            sqlx::query(
                "SELECT uuid, tenant_id, site_id, config_name, config_type, is_active, status
                 FROM web_nginx_config WHERE tenant_id = $1 AND uuid = $2",
            )
            .bind(tenant_id)
            .bind(config_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| store_error("retrieve web_nginx_config", error))?
        } else {
            sqlx::query(
                "SELECT uuid, tenant_id, site_id, config_name, config_type, is_active, status
                 FROM web_nginx_config WHERE uuid = $1",
            )
            .bind(config_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| store_error("retrieve web_nginx_config", error))?
        }
        .ok_or_else(|| WebServiceError::not_found("nginx config not found"))?;

        let row_tenant_id: i64 = row
            .try_get("tenant_id")
            .map_err(|error| store_error("retrieve web_nginx_config tenant_id", error))?;
        map_nginx_config_row(&self.pool, row_tenant_id, &row)
            .await
            .map_err(|error| WebServiceError::Internal(error.to_string()))
    }

    pub(super) async fn update_nginx_config_repo(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
        request: &UpdateNginxConfigRequest,
    ) -> WebServiceResult<NginxConfigResponse> {
        let existing = self
            .retrieve_nginx_config_repo(tenant_id, config_id)
            .await?;
        let row = if let Some(tenant_id) = tenant_id {
            sqlx::query(
                "SELECT config_name, config_content FROM web_nginx_config
                 WHERE tenant_id = $1 AND uuid = $2",
            )
            .bind(tenant_id)
            .bind(config_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| store_error("load web_nginx_config for update", error))?
        } else {
            sqlx::query("SELECT config_name, config_content FROM web_nginx_config WHERE uuid = $1")
                .bind(config_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|error| store_error("load web_nginx_config for update", error))?
        }
        .ok_or_else(|| WebServiceError::not_found("nginx config not found"))?;

        let config_name = request
            .config_name
            .as_ref()
            .cloned()
            .or_else(|| row.try_get("config_name").ok())
            .unwrap_or(existing.config_name);
        let config_content = request
            .config_content
            .as_ref()
            .cloned()
            .or_else(|| row.try_get("config_content").ok())
            .unwrap_or_default();
        let config_hash = sha256_hex(&config_content);
        let now = now_rfc3339();

        let result = if let Some(tenant_id) = tenant_id {
            sqlx::query(
                "UPDATE web_nginx_config
                 SET config_name = $3, config_content = $4, config_hash = $5, updated_at = $6, version = version + 1
                 WHERE tenant_id = $1 AND uuid = $2",
            )
            .bind(tenant_id)
            .bind(config_id)
            .bind(&config_name)
            .bind(&config_content)
            .bind(&config_hash)
            .bind(&now)
            .execute(&self.pool)
            .await
            .map_err(|error| store_error("update web_nginx_config", error))?
        } else {
            sqlx::query(
                "UPDATE web_nginx_config
                 SET config_name = $2, config_content = $3, config_hash = $4, updated_at = $5, version = version + 1
                 WHERE uuid = $1",
            )
            .bind(config_id)
            .bind(&config_name)
            .bind(&config_content)
            .bind(&config_hash)
            .bind(&now)
            .execute(&self.pool)
            .await
            .map_err(|error| store_error("update web_nginx_config", error))?
        };

        if result.rows_affected() == 0 {
            return Err(WebServiceError::not_found("nginx config not found"));
        }

        self.retrieve_nginx_config_repo(tenant_id, config_id).await
    }

    pub(super) async fn load_nginx_config_content_repo(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> WebServiceResult<String> {
        let row = if let Some(tenant_id) = tenant_id {
            sqlx::query(
                "SELECT config_content FROM web_nginx_config WHERE tenant_id = $1 AND uuid = $2",
            )
            .bind(tenant_id)
            .bind(config_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| store_error("load web_nginx_config content", error))?
        } else {
            sqlx::query("SELECT config_content FROM web_nginx_config WHERE uuid = $1")
                .bind(config_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|error| store_error("load web_nginx_config content", error))?
        }
        .ok_or_else(|| WebServiceError::not_found("nginx config not found"))?;

        row.try_get("config_content")
            .map_err(|error| store_error("load web_nginx_config content column", error))
    }

    pub(super) async fn validate_nginx_config_repo(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> WebServiceResult<NginxValidateResponse> {
        let row = if let Some(tenant_id) = tenant_id {
            sqlx::query(
                "SELECT config_content FROM web_nginx_config WHERE tenant_id = $1 AND uuid = $2",
            )
            .bind(tenant_id)
            .bind(config_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| store_error("validate web_nginx_config lookup", error))?
        } else {
            sqlx::query("SELECT config_content FROM web_nginx_config WHERE uuid = $1")
                .bind(config_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|error| store_error("validate web_nginx_config lookup", error))?
        }
        .ok_or_else(|| WebServiceError::not_found("nginx config not found"))?;

        let content: String = row
            .try_get("config_content")
            .map_err(|error| store_error("validate web_nginx_config content", error))?;
        let valid = !content.trim().is_empty();
        Ok(NginxValidateResponse {
            valid,
            message: if valid {
                None
            } else {
                Some("config content is empty".to_string())
            },
        })
    }

    pub(super) async fn web_nginx_config_repo(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> WebServiceResult<NginxConfigResponse> {
        let existing = self
            .retrieve_nginx_config_repo(tenant_id, config_id)
            .await?;
        let site_internal_id =
            resolve_site_internal_id(&self.pool, tenant_id.unwrap_or(0), &existing.site_id).await?;
        let now = now_rfc3339();

        // 事务边界：停用旧 active config + 激活目标 config 必须原子完成，
        // 避免停用成功但激活失败导致站点丢失生效配置。
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|error| store_error("begin deploy web_nginx_config transaction", error))?;

        sqlx::query(
            "UPDATE web_nginx_config SET is_active = 0, updated_at = $2, version = version + 1
             WHERE site_id = $1 AND is_active = 1",
        )
        .bind(site_internal_id)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("deactivate web_nginx_config", error))?;

        let result = if let Some(tenant_id) = tenant_id {
            sqlx::query(
                "UPDATE web_nginx_config
                 SET is_active = 1, status = 1, deployed_at = $3, updated_at = $3, version = version + 1
                 WHERE tenant_id = $1 AND uuid = $2",
            )
            .bind(tenant_id)
            .bind(config_id)
            .bind(&now)
            .execute(&mut *tx)
            .await
            .map_err(|error| store_error("Web web_nginx_config", error))?
        } else {
            sqlx::query(
                "UPDATE web_nginx_config
                 SET is_active = 1, status = 1, deployed_at = $2, updated_at = $2, version = version + 1
                 WHERE uuid = $1",
            )
            .bind(config_id)
            .bind(&now)
            .execute(&mut *tx)
            .await
            .map_err(|error| store_error("Web web_nginx_config", error))?
        };

        tx.commit()
            .await
            .map_err(|error| store_error("commit deploy web_nginx_config transaction", error))?;

        if result.rows_affected() == 0 {
            return Err(WebServiceError::not_found("nginx config not found"));
        }

        self.retrieve_nginx_config_repo(tenant_id, config_id).await
    }

    pub(super) async fn reload_nginx_repo(&self) -> WebServiceResult<NginxReloadResponse> {
        Ok(NginxReloadResponse { reloaded: true })
    }

    pub(super) async fn retrieve_nginx_status_repo(
        &self,
        tenant_id: Option<i64>,
    ) -> WebServiceResult<NginxStatusResponse> {
        let active_configs = if let Some(tenant_id) = tenant_id {
            let row = sqlx::query(
                "SELECT COUNT(*) AS total FROM web_nginx_config
                 WHERE tenant_id = $1 AND is_active = 1 AND status = 1",
            )
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|error| store_error("count active web_nginx_config", error))?;
            row.try_get::<i64, _>("total").unwrap_or(0)
        } else {
            let row = sqlx::query(
                "SELECT COUNT(*) AS total FROM web_nginx_config WHERE is_active = 1 AND status = 1",
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|error| store_error("count active web_nginx_config", error))?;
            row.try_get::<i64, _>("total").unwrap_or(0)
        };

        Ok(NginxStatusResponse {
            running: active_configs > 0,
            active_configs,
        })
    }

    pub(super) async fn resolve_site_primary_hostname_repo(
        &self,
        tenant_id: i64,
        site_uuid: &str,
    ) -> WebServiceResult<String> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_uuid).await?;
        let row = sqlx::query(
            "SELECT hostname FROM web_domain
             WHERE tenant_id = $1 AND site_id = $2 AND deleted_at IS NULL
             ORDER BY is_primary DESC, created_at ASC LIMIT 1",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("resolve primary hostname", error))?
        .ok_or_else(|| WebServiceError::not_found("domain not found for site"))?;

        row.try_get("hostname")
            .map_err(|error| store_error("resolve primary hostname column", error))
    }
}

enum BindValue {
    I64(i64),
    I32(i32),
    Bool(bool),
}

fn apply_binds<'q>(
    mut query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>,
    binds: &[BindValue],
) -> sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>> {
    for value in binds {
        query = match value {
            BindValue::I64(value) => query.bind(*value),
            BindValue::I32(value) => query.bind(*value),
            BindValue::Bool(value) => query.bind(*value),
        };
    }
    query
}

async fn map_nginx_config_row(
    pool: &sqlx::AnyPool,
    tenant_id: i64,
    row: &AnyRow,
) -> Result<NginxConfigResponse, sqlx::Error> {
    let site_internal_id: i64 = row.try_get("site_id")?;
    let site_uuid = resolve_site_uuid(pool, tenant_id, site_internal_id)
        .await
        .map_err(|error| sqlx::Error::Decode(error.to_string().into()))?;

    Ok(NginxConfigResponse {
        id: row.try_get("uuid")?,
        site_id: site_uuid,
        config_name: row.try_get("config_name")?,
        config_type: row.try_get("config_type")?,
        is_active: bool_from_row(row, "is_active")?,
        status: row.try_get("status")?,
    })
}
