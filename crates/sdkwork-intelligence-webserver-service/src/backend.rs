//! Backend-api service surface implementation.

use async_trait::async_trait;
use sdkwork_webserver_contract::{
    CreateNginxConfigRequest, CreateServerRequest, ListNginxConfigsQuery, UpdateNginxConfigRequest,
    WebBackendApi, WebBackendRequestContext, WebServiceError, WebServiceResult,
};

use crate::WebService;

impl WebService {
    /// 统一的 fail-closed 租户上下文校验。
    ///
    /// 所有 backend-api 操作（读与写）都必须携带有效 tenant_id（>0），
    /// 防止 `tenant_id=None` 时跨租户读写数据。
    /// 平台级跨租户管理操作应通过独立 platform-admin 鉴权链路实现，不复用此通道。
    fn require_backend_tenant(context: &WebBackendRequestContext) -> WebServiceResult<i64> {
        context
            .tenant_id
            .filter(|tenant_id| *tenant_id > 0)
            .ok_or(WebServiceError::validation(
                "tenant context is required for backend operations",
            ))
    }
}

#[async_trait]
impl WebBackendApi for WebService {
    async fn list_nginx_configs(
        &self,
        context: &WebBackendRequestContext,
        query: &ListNginxConfigsQuery,
    ) -> WebServiceResult<sdkwork_webserver_contract::NginxConfigPage> {
        let tenant_id = Self::require_backend_tenant(context)?;
        self.repository
            .list_nginx_configs(Some(tenant_id), query)
            .await
    }

    async fn create_nginx_config(
        &self,
        context: &WebBackendRequestContext,
        request: &CreateNginxConfigRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::NginxConfigResponse> {
        let tenant_id = Self::require_backend_tenant(context)?;
        self.repository
            .create_nginx_config(tenant_id, request)
            .await
    }

    async fn retrieve_nginx_config(
        &self,
        context: &WebBackendRequestContext,
        config_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::NginxConfigResponse> {
        let tenant_id = Self::require_backend_tenant(context)?;
        self.repository
            .retrieve_nginx_config(Some(tenant_id), config_id)
            .await
    }

    async fn update_nginx_config(
        &self,
        context: &WebBackendRequestContext,
        config_id: &str,
        request: &UpdateNginxConfigRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::NginxConfigResponse> {
        let tenant_id = Self::require_backend_tenant(context)?;
        self.repository
            .update_nginx_config(Some(tenant_id), config_id, request)
            .await
    }

    async fn validate_nginx_config(
        &self,
        context: &WebBackendRequestContext,
        config_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::NginxValidateResponse> {
        let tenant_id = Self::require_backend_tenant(context)?;
        let content = self
            .repository
            .load_nginx_config_content(Some(tenant_id), config_id)
            .await?;
        match self.validate_nginx_content(&content).await {
            Ok(()) => Ok(sdkwork_webserver_contract::NginxValidateResponse {
                valid: true,
                message: None,
            }),
            Err(error) => Ok(sdkwork_webserver_contract::NginxValidateResponse {
                valid: false,
                message: Some(error.to_string()),
            }),
        }
    }

    async fn web_nginx_config(
        &self,
        context: &WebBackendRequestContext,
        config_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::NginxConfigResponse> {
        let tenant_id = Self::require_backend_tenant(context)?;
        let candidate = self
            .repository
            .retrieve_nginx_config(Some(tenant_id), config_id)
            .await?;
        let domain = self
            .repository
            .resolve_site_primary_hostname(tenant_id, &candidate.site_id)
            .await?;
        let content = self
            .repository
            .load_nginx_config_content(Some(tenant_id), config_id)
            .await?;
        self.validate_nginx_content(&content).await?;

        let response = self
            .repository
            .web_nginx_config(Some(tenant_id), config_id)
            .await?;
        self.deploy_nginx_site(&domain, &content).await?;
        self.reload_nginx_runtime().await?;

        Ok(response)
    }

    async fn reload_nginx(
        &self,
        _context: &WebBackendRequestContext,
    ) -> WebServiceResult<sdkwork_webserver_contract::NginxReloadResponse> {
        self.reload_nginx_runtime().await?;
        Ok(sdkwork_webserver_contract::NginxReloadResponse { reloaded: true })
    }

    async fn retrieve_nginx_status(
        &self,
        context: &WebBackendRequestContext,
    ) -> WebServiceResult<sdkwork_webserver_contract::NginxStatusResponse> {
        let tenant_id = Self::require_backend_tenant(context)?;
        self.repository.retrieve_nginx_status(Some(tenant_id)).await
    }

    async fn list_servers(
        &self,
        context: &WebBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<sdkwork_webserver_contract::ServerPage> {
        let tenant_id = Self::require_backend_tenant(context)?;
        self.repository
            .list_servers(tenant_id, page, page_size)
            .await
    }

    async fn create_server(
        &self,
        context: &WebBackendRequestContext,
        request: &CreateServerRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::CreateServerResponse> {
        let tenant_id = Self::require_backend_tenant(context)?;
        validate_tenant_scope_hash(&request.tenant_scope_hash)?;
        self.repository.create_server(tenant_id, request).await
    }

    async fn list_audit_logs(
        &self,
        context: &WebBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<sdkwork_webserver_contract::AuditLogPage> {
        let tenant_id = Self::require_backend_tenant(context)?;
        self.repository
            .list_audit_logs(Some(tenant_id), page, page_size)
            .await
    }
}

fn validate_tenant_scope_hash(value: &str) -> WebServiceResult<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(WebServiceError::validation(
            "tenantScopeHash must be a lowercase SHA-256 digest",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_tenant_scope_hash;

    #[test]
    fn tenant_scope_hash_is_exact_lowercase_sha256_shape() {
        validate_tenant_scope_hash(&"a".repeat(64)).unwrap();
        for invalid in ["a".repeat(63), "A".repeat(64), "g".repeat(64)] {
            assert!(validate_tenant_scope_hash(&invalid).is_err());
        }
    }
}
