use super::{EngineRow, WebRepository};
use sdkwork_webserver_contract::{
    CreateServerRequest, CreateServerResponse, ServerPage, ServerResponse, WebServiceError,
    WebServiceResult,
};
use serde_json::json;
use sqlx::Row;

use super::agents::{generate_agent_token, hash_agent_token, parse_last_heartbeat_at};
use super::support::{
    instant_write_expression, json_from_row, json_write_expression, new_uuid, next_id, now_rfc3339,
    pagination, store_error,
};

impl WebRepository {
    pub(super) async fn list_servers_repo(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<ServerPage> {
        let (_page, page_size, offset) = pagination(page, page_size);

        let count_row =
            sqlx::query("SELECT COUNT(*) AS total FROM web_server WHERE tenant_id = $1")
                .bind(tenant_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|error| store_error("count web_server", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let rows = sqlx::query(
            "SELECT uuid, name, host, tenant_scope_hash, ssh_port, status,
                    CAST(metadata AS TEXT) AS metadata,
                    CAST(created_at AS TEXT) AS created_at
             FROM web_server
             WHERE tenant_id = $1
             ORDER BY updated_at DESC, id DESC LIMIT $2 OFFSET $3",
        )
        .bind(tenant_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("list web_server", error))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_server_row(row).map_err(|error| {
                WebServiceError::Internal(format!("map web_server row: {error}"))
            })?);
        }

        Ok(ServerPage { items, total })
    }

    pub(super) async fn create_server_repo(
        &self,
        tenant_id: i64,
        request: &CreateServerRequest,
    ) -> WebServiceResult<CreateServerResponse> {
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();
        let agent_token = generate_agent_token();
        let metadata = json!({
            "agentTokenHash": hash_agent_token(&agent_token),
        });
        let engine = self.database_engine().await?;
        let metadata_expression = json_write_expression(engine, "$8");
        let now_expression = instant_write_expression(engine, "$9");
        let insert_sql = format!(
            "INSERT INTO web_server (
                id, uuid, tenant_id, name, host, tenant_scope_hash, ssh_port, status, metadata,
                created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, 0, {metadata_expression},
                {now_expression}, {now_expression}, 0
             )"
        );

        sqlx::query(&insert_sql)
            .bind(id)
            .bind(&uuid)
            .bind(tenant_id)
            .bind(&request.name)
            .bind(&request.host)
            .bind(&request.tenant_scope_hash)
            .bind(request.ssh_port)
            .bind(metadata.to_string())
            .bind(&now)
            .execute(&self.pool)
            .await
            .map_err(|error| store_error("insert web_server", error))?;

        Ok(CreateServerResponse {
            server: ServerResponse {
                id: uuid,
                name: request.name.clone(),
                host: request.host.clone(),
                tenant_scope_hash: request.tenant_scope_hash.clone(),
                ssh_port: request.ssh_port,
                status: 0,
                last_heartbeat_at: None,
                created_at: now,
            },
            agent_token,
        })
    }
}

fn map_server_row(row: &EngineRow) -> Result<ServerResponse, sqlx::Error> {
    let metadata_raw = json_from_row(row, "metadata")?
        .unwrap_or_else(|| json!({}))
        .to_string();
    Ok(ServerResponse {
        id: row.try_get("uuid")?,
        name: row.try_get("name")?,
        host: row.try_get("host")?,
        tenant_scope_hash: row.try_get("tenant_scope_hash")?,
        ssh_port: row.try_get("ssh_port")?,
        status: row.try_get("status")?,
        last_heartbeat_at: parse_last_heartbeat_at(&metadata_raw),
        created_at: row.try_get("created_at")?,
    })
}
