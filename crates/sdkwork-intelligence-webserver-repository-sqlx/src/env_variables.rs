use sdkwork_webserver_contract::{
    CreateEnvVariableRequest, EnvVariablePage, EnvVariableResponse, WebServiceError,
    WebServiceResult,
};
use sqlx::{any::AnyRow, Row};

use crate::support::{
    bool_from_row, new_uuid, next_id, now_rfc3339, resolve_site_internal_id, store_error,
};
use crate::WebRepository;

impl WebRepository {
    pub(super) async fn list_env_variables_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        environment: Option<&str>,
    ) -> WebServiceResult<EnvVariablePage> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;

        let (count_row, rows) = if let Some(environment) = environment {
            let count_row = sqlx::query(
                "SELECT COUNT(*) AS total FROM web_env_variable
                 WHERE tenant_id = $1 AND site_id = $2 AND environment = $3 AND status = 1",
            )
            .bind(tenant_id)
            .bind(site_internal_id)
            .bind(environment)
            .fetch_one(&self.pool)
            .await
            .map_err(|error| store_error("count web_env_variable", error))?;

            let rows = sqlx::query(
                "SELECT uuid, key, value_encrypted, environment, is_secret
                 FROM web_env_variable
                 WHERE tenant_id = $1 AND site_id = $2 AND environment = $3 AND status = 1
                 ORDER BY key ASC",
            )
            .bind(tenant_id)
            .bind(site_internal_id)
            .bind(environment)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("list web_env_variable", error))?;

            (count_row, rows)
        } else {
            let count_row = sqlx::query(
                "SELECT COUNT(*) AS total FROM web_env_variable
                 WHERE tenant_id = $1 AND site_id = $2 AND status = 1",
            )
            .bind(tenant_id)
            .bind(site_internal_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|error| store_error("count web_env_variable", error))?;

            let rows = sqlx::query(
                "SELECT uuid, key, value_encrypted, environment, is_secret
                 FROM web_env_variable
                 WHERE tenant_id = $1 AND site_id = $2 AND status = 1
                 ORDER BY environment ASC, key ASC",
            )
            .bind(tenant_id)
            .bind(site_internal_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("list web_env_variable", error))?;

            (count_row, rows)
        };

        let total: i64 = count_row.try_get("total").unwrap_or(0);
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_env_variable_row(row).map_err(|error| {
                WebServiceError::Internal(format!("map web_env_variable row: {error}"))
            })?);
        }

        Ok(EnvVariablePage { items, total })
    }

    pub(super) async fn create_env_variable_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateEnvVariableRequest,
    ) -> WebServiceResult<EnvVariableResponse> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();

        sqlx::query(
            "INSERT INTO web_env_variable (
                id, uuid, tenant_id, site_id, environment, key, value_encrypted, is_secret,
                status, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, 1, $9, $9, 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(&request.environment)
        .bind(&request.key)
        .bind(&request.value)
        .bind(request.is_secret)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert web_env_variable", error))?;

        Ok(EnvVariableResponse {
            id: uuid,
            key: request.key.clone(),
            value: request.value.clone(),
            environment: request.environment.clone(),
            is_secret: request.is_secret,
        })
    }
}

fn map_env_variable_row(row: &AnyRow) -> Result<EnvVariableResponse, sqlx::Error> {
    Ok(EnvVariableResponse {
        id: row.try_get("uuid")?,
        key: row.try_get("key")?,
        value: row.try_get("value_encrypted")?,
        environment: row.try_get("environment")?,
        is_secret: bool_from_row(row, "is_secret")?,
    })
}
