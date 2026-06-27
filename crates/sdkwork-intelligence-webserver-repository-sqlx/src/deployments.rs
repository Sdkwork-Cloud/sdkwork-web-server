use sdkwork_webserver_contract::{
    CreateDeploymentRequest, DeploymentPage, DeploymentResponse, WebServiceError, WebServiceResult,
};
use sqlx::{any::AnyRow, Row};

use crate::support::{
    new_uuid, next_id, now_rfc3339, pagination, resolve_site_internal_id, resolve_site_uuid,
    store_error,
};
use crate::WebRepository;

impl WebRepository {
    pub(super) async fn list_deployments_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
        status: Option<i32>,
    ) -> WebServiceResult<DeploymentPage> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let (page, page_size, offset) = pagination(page, page_size);

        let (count_row, rows) = if let Some(status) = status {
            let count_row = sqlx::query(
                "SELECT COUNT(*) AS total FROM web_deployment
                 WHERE tenant_id = $1 AND site_id = $2 AND status = $3",
            )
            .bind(tenant_id)
            .bind(site_internal_id)
            .bind(status)
            .fetch_one(&self.pool)
            .await
            .map_err(|error| store_error("count web_deployment", error))?;

            let rows = sqlx::query(
                "SELECT uuid, site_id, status, deploy_type, created_at
                 FROM web_deployment
                 WHERE tenant_id = $1 AND site_id = $2 AND status = $3
                 ORDER BY created_at DESC LIMIT $4 OFFSET $5",
            )
            .bind(tenant_id)
            .bind(site_internal_id)
            .bind(status)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("list web_deployment", error))?;

            (count_row, rows)
        } else {
            let count_row = sqlx::query(
                "SELECT COUNT(*) AS total FROM web_deployment
                 WHERE tenant_id = $1 AND site_id = $2",
            )
            .bind(tenant_id)
            .bind(site_internal_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|error| store_error("count web_deployment", error))?;

            let rows = sqlx::query(
                "SELECT uuid, site_id, status, deploy_type, created_at
                 FROM web_deployment
                 WHERE tenant_id = $1 AND site_id = $2
                 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
            )
            .bind(tenant_id)
            .bind(site_internal_id)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("list web_deployment", error))?;

            (count_row, rows)
        };

        let total: i64 = count_row.try_get("total").unwrap_or(0);
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(
                map_deployment_row(&self.pool, tenant_id, row)
                    .await
                    .map_err(|error| {
                        WebServiceError::Internal(format!("map web_deployment row: {error}"))
                    })?,
            );
        }

        Ok(DeploymentPage {
            items,
            total,
            page,
            page_size,
        })
    }

    pub(super) async fn create_deployment_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        actor_id: Option<i64>,
        request: &CreateDeploymentRequest,
    ) -> WebServiceResult<DeploymentResponse> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;

        // 幂等性：如果客户端提供了非空 idempotency_key，
        // 先查找是否已存在相同 (tenant_id, idempotency_key) 的 deployment。
        // 存在则直接返回已创建的记录，保证网络重试不会产生重复部署。
        let idempotency_key = request
            .idempotency_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if let Some(key) = idempotency_key {
            if let Some(existing) = self
                .find_deployment_by_idempotency_repo(tenant_id, site_internal_id, key)
                .await?
            {
                return Ok(existing);
            }
        }

        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();
        let environment = request
            .environment
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("production");

        sqlx::query(
            "INSERT INTO web_deployment (
                id, uuid, tenant_id, user_id, site_id, deploy_type, environment, status,
                idempotency_key, metadata, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, 0, $8, '{}', $9, $9, 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(actor_id)
        .bind(site_internal_id)
        .bind(request.deploy_type)
        .bind(environment)
        .bind(idempotency_key)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert web_deployment", error))?;

        self.retrieve_deployment_repo(tenant_id, site_id, &uuid)
            .await
    }

    /// 通过 (tenant_id, site_id, idempotency_key) 查找已存在的 deployment。
    /// 用于 create_deployment 的幂等性检查。
    async fn find_deployment_by_idempotency_repo(
        &self,
        tenant_id: i64,
        site_internal_id: i64,
        idempotency_key: &str,
    ) -> WebServiceResult<Option<DeploymentResponse>> {
        let row = sqlx::query(
            "SELECT uuid, site_id, status, deploy_type, created_at
             FROM web_deployment
             WHERE tenant_id = $1 AND site_id = $2 AND idempotency_key = $3",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("find web_deployment by idempotency_key", error))?;

        match row {
            Some(row) => Ok(Some(
                map_deployment_row(&self.pool, tenant_id, &row)
                    .await
                    .map_err(|error| {
                        WebServiceError::Internal(format!("map web_deployment row: {error}"))
                    })?,
            )),
            None => Ok(None),
        }
    }

    pub(super) async fn retrieve_deployment_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        deployment_id: &str,
    ) -> WebServiceResult<DeploymentResponse> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let row = sqlx::query(
            "SELECT uuid, site_id, status, deploy_type, created_at
             FROM web_deployment
             WHERE tenant_id = $1 AND site_id = $2 AND uuid = $3",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(deployment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("retrieve web_deployment", error))?
        .ok_or_else(|| WebServiceError::not_found("deployment not found"))?;

        map_deployment_row(&self.pool, tenant_id, &row)
            .await
            .map_err(|error| WebServiceError::Internal(error.to_string()))
    }

    pub(super) async fn rollback_deployment_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        deployment_id: &str,
        actor_id: Option<i64>,
    ) -> WebServiceResult<DeploymentResponse> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let source = sqlx::query(
            "SELECT id, deploy_type, environment FROM web_deployment
             WHERE tenant_id = $1 AND site_id = $2 AND uuid = $3",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(deployment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("rollback web_deployment lookup", error))?
        .ok_or_else(|| WebServiceError::not_found("deployment not found"))?;

        let source_id: i64 = source
            .try_get("id")
            .map_err(|error| store_error("rollback web_deployment source id", error))?;
        let deploy_type: i32 = source
            .try_get("deploy_type")
            .map_err(|error| store_error("rollback web_deployment deploy_type", error))?;
        let environment: String = source
            .try_get("environment")
            .map_err(|error| store_error("rollback web_deployment environment", error))?;
        let now = now_rfc3339();
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();

        // 事务边界：标记源 deployment 为已回滚 + 创建 rollback 记录必须原子完成，
        // 避免标记成功但记录创建失败导致状态不一致。
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|error| store_error("begin rollback web_deployment transaction", error))?;

        sqlx::query(
            "UPDATE web_deployment
             SET status = 5, updated_at = $4, version = version + 1
             WHERE tenant_id = $1 AND site_id = $2 AND uuid = $3",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(deployment_id)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("mark web_deployment rolled back", error))?;

        sqlx::query(
            "INSERT INTO web_deployment (
                id, uuid, tenant_id, user_id, site_id, deploy_type, environment, status,
                rollback_from, metadata, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, 0, $8, '{}', $9, $9, 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(actor_id)
        .bind(site_internal_id)
        .bind(deploy_type)
        .bind(&environment)
        .bind(source_id)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("insert rollback web_deployment", error))?;

        tx.commit()
            .await
            .map_err(|error| store_error("commit rollback web_deployment transaction", error))?;

        self.retrieve_deployment_repo(tenant_id, site_id, &uuid)
            .await
    }
}

async fn map_deployment_row(
    pool: &sqlx::AnyPool,
    tenant_id: i64,
    row: &AnyRow,
) -> Result<DeploymentResponse, sqlx::Error> {
    let site_internal_id: i64 = row.try_get("site_id")?;
    let site_uuid = resolve_site_uuid(pool, tenant_id, site_internal_id)
        .await
        .map_err(|error| sqlx::Error::Decode(error.to_string().into()))?;

    Ok(DeploymentResponse {
        id: row.try_get("uuid")?,
        site_id: site_uuid,
        status: row.try_get("status")?,
        deploy_type: row.try_get("deploy_type")?,
        created_at: row.try_get("created_at")?,
    })
}
