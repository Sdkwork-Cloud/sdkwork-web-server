use sdkwork_utils_rust::aes_gcm_encrypt;
use sdkwork_webserver_contract::{
    CreateEnvVariableRequest, EnvVariablePage, EnvVariableResponse, WebServiceError,
    WebServiceResult,
};
use sqlx::{any::AnyRow, Row};

use crate::support::{
    bool_from_row, new_uuid, next_id, now_rfc3339, resolve_site_internal_id, store_error,
};
use crate::WebRepository;

/// 机密值在 list/retrieve 响应中的掩码占位符。
/// 真实值仅通过 create 接口接收并加密落库，永不在查询响应中返回明文。
const SECRET_VALUE_MASK: &str = "***";

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

        // 机密值必须加密后落库，非机密值原样存储以保持可读性与查询效率。
        let stored_value = if request.is_secret {
            aes_gcm_encrypt(self.secret_key(), request.value.as_bytes()).map_err(|error| {
                WebServiceError::Internal(format!("encrypt env variable: {error}"))
            })?
        } else {
            request.value.clone()
        };

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
        .bind(&stored_value)
        .bind(request.is_secret)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert web_env_variable", error))?;

        // 响应中机密值返回掩码，不回传明文/密文，避免泄漏。
        Ok(EnvVariableResponse {
            id: uuid,
            key: request.key.clone(),
            value: if request.is_secret {
                SECRET_VALUE_MASK.to_string()
            } else {
                request.value.clone()
            },
            environment: request.environment.clone(),
            is_secret: request.is_secret,
        })
    }
}

fn map_env_variable_row(row: &AnyRow) -> Result<EnvVariableResponse, sqlx::Error> {
    let is_secret = bool_from_row(row, "is_secret")?;
    Ok(EnvVariableResponse {
        id: row.try_get("uuid")?,
        key: row.try_get("key")?,
        // 机密值在查询响应中始终返回掩码，永不明文回传。
        // value_encrypted 列对 is_secret=true 存储的是 base64(nonce||ciphertext)，不可直接展示。
        value: if is_secret {
            SECRET_VALUE_MASK.to_string()
        } else {
            row.try_get("value_encrypted")?
        },
        environment: row.try_get("environment")?,
        is_secret,
    })
}
