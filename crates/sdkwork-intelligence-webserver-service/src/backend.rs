//! Backend-api service surface implementation.

use async_trait::async_trait;
use sdkwork_webserver_contract::{
    CreateNginxConfigRequest, CreateServerRequest, ListNginxConfigsQuery, UpdateNginxConfigRequest,
    WebBackendApi, WebBackendRequestContext, WebServiceError, WebServiceResult,
};

use crate::WebService;

impl WebService {
    fn backend_tenant_scope(context: &WebBackendRequestContext) -> WebServiceResult<Option<i64>> {
        Ok(context.tenant_id)
    }

    fn backend_write_tenant(context: &WebBackendRequestContext) -> WebServiceResult<i64> {
        context
            .tenant_id
            .filter(|tenant_id| *tenant_id > 0)
            .ok_or(WebServiceError::validation(
                "tenant context is required for backend write operations",
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
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository.list_nginx_configs(tenant_id, query).await
    }

    async fn create_nginx_config(
        &self,
        context: &WebBackendRequestContext,
        request: &CreateNginxConfigRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::NginxConfigResponse> {
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository
            .create_nginx_config(tenant_id, request)
            .await
    }

    async fn retrieve_nginx_config(
        &self,
        context: &WebBackendRequestContext,
        config_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::NginxConfigResponse> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository
            .retrieve_nginx_config(tenant_id, config_id)
            .await
    }

    async fn update_nginx_config(
        &self,
        context: &WebBackendRequestContext,
        config_id: &str,
        request: &UpdateNginxConfigRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::NginxConfigResponse> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository
            .update_nginx_config(tenant_id, config_id, request)
            .await
    }

    async fn validate_nginx_config(
        &self,
        context: &WebBackendRequestContext,
        config_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::NginxValidateResponse> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        let content = self
            .repository
            .load_nginx_config_content(tenant_id, config_id)
            .await?;
        match self.validate_nginx_content(&content) {
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
        let tenant_id = Self::backend_tenant_scope(context)?;
        let response = self
            .repository
            .web_nginx_config(tenant_id, config_id)
            .await?;

        if let Some(tenant_id) = tenant_id {
            if let Ok(content) = self
                .repository
                .load_nginx_config_content(Some(tenant_id), config_id)
                .await
            {
                if let Ok(domain) = self
                    .repository
                    .resolve_site_primary_hostname(tenant_id, &response.site_id)
                    .await
                {
                    let _ = self.deploy_nginx_site(&domain, &content);
                    let _ = self.reload_nginx_runtime();
                }
            }
        }

        Ok(response)
    }

    async fn reload_nginx(
        &self,
        _context: &WebBackendRequestContext,
    ) -> WebServiceResult<sdkwork_webserver_contract::NginxReloadResponse> {
        self.reload_nginx_runtime()?;
        Ok(sdkwork_webserver_contract::NginxReloadResponse { reloaded: true })
    }

    async fn retrieve_nginx_status(
        &self,
        context: &WebBackendRequestContext,
    ) -> WebServiceResult<sdkwork_webserver_contract::NginxStatusResponse> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository.retrieve_nginx_status(tenant_id).await
    }

    async fn list_servers(
        &self,
        context: &WebBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<sdkwork_webserver_contract::ServerPage> {
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository
            .list_servers(tenant_id, page, page_size)
            .await
    }

    async fn create_server(
        &self,
        context: &WebBackendRequestContext,
        request: &CreateServerRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::CreateServerResponse> {
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository.create_server(tenant_id, request).await
    }

    async fn list_audit_logs(
        &self,
        context: &WebBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<sdkwork_webserver_contract::AuditLogPage> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository
            .list_audit_logs(tenant_id, page, page_size)
            .await
    }
}
