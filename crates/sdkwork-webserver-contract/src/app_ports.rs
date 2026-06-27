use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::dto::*;
use crate::problem::WebServiceResult;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebAppRequestContext {
    pub tenant_id: i64,
    pub actor_id: Option<i64>,
    pub organization_id: Option<i64>,
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebBackendRequestContext {
    pub operator_id: Option<i64>,
    pub tenant_id: Option<i64>,
    /// Raw principal subject identifier (server UUID for agent-token routes, user_id string for dual-token).
    /// Present when the framework resolves a principal; absent for anonymous/public contexts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_id: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ListSitesQuery {
    #[serde(default = "crate::dto::default_page")]
    pub page: i32,
    #[serde(default = "crate::dto::default_page_size", rename = "pageSize")]
    pub page_size: i32,
    pub status: Option<i32>,
    #[serde(rename = "siteType")]
    pub site_type: Option<i32>,
    pub keyword: Option<String>,
}

#[async_trait]
pub trait WebAppApi: Send + Sync {
    async fn list_sites(
        &self,
        context: &WebAppRequestContext,
        query: &ListSitesQuery,
    ) -> WebServiceResult<SitePage>;

    async fn create_site(
        &self,
        context: &WebAppRequestContext,
        request: &CreateSiteRequest,
    ) -> WebServiceResult<SiteResponse>;

    async fn retrieve_site(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
    ) -> WebServiceResult<SiteResponse>;

    async fn update_site(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        request: &UpdateSiteRequest,
    ) -> WebServiceResult<SiteResponse>;

    async fn delete_site(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
    ) -> WebServiceResult<()>;

    async fn activate_site(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
    ) -> WebServiceResult<SiteResponse>;

    async fn pause_site(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
    ) -> WebServiceResult<SiteResponse>;

    async fn list_domains(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<DomainPage>;

    async fn create_domain(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        request: &CreateDomainRequest,
    ) -> WebServiceResult<DomainResponse>;

    async fn retrieve_domain(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        domain_id: &str,
    ) -> WebServiceResult<DomainResponse>;

    async fn delete_domain(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        domain_id: &str,
    ) -> WebServiceResult<()>;

    async fn verify_domain(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        domain_id: &str,
    ) -> WebServiceResult<DomainVerifyResponse>;

    async fn list_deployments(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        page: i32,
        page_size: i32,
        status: Option<i32>,
    ) -> WebServiceResult<DeploymentPage>;

    async fn create_deployment(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        request: &CreateDeploymentRequest,
    ) -> WebServiceResult<DeploymentResponse>;

    async fn retrieve_deployment(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        deployment_id: &str,
    ) -> WebServiceResult<DeploymentResponse>;

    async fn rollback_deployment(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        deployment_id: &str,
    ) -> WebServiceResult<DeploymentResponse>;

    async fn list_env_variables(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        environment: Option<&str>,
    ) -> WebServiceResult<EnvVariablePage>;

    async fn create_env_variable(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        request: &CreateEnvVariableRequest,
    ) -> WebServiceResult<EnvVariableResponse>;

    async fn list_certificates(
        &self,
        context: &WebAppRequestContext,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<CertificatePage>;

    async fn create_certificate(
        &self,
        context: &WebAppRequestContext,
        request: &CreateCertificateRequest,
    ) -> WebServiceResult<CertificateResponse>;

    async fn list_health_checks(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
    ) -> WebServiceResult<HealthCheckPage>;

    async fn create_health_check(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        request: &CreateHealthCheckRequest,
    ) -> WebServiceResult<HealthCheckResponse>;
}

#[async_trait]
pub trait WebBackendApi: Send + Sync {
    async fn list_nginx_configs(
        &self,
        context: &WebBackendRequestContext,
        query: &ListNginxConfigsQuery,
    ) -> WebServiceResult<NginxConfigPage>;

    async fn create_nginx_config(
        &self,
        context: &WebBackendRequestContext,
        request: &CreateNginxConfigRequest,
    ) -> WebServiceResult<NginxConfigResponse>;

    async fn retrieve_nginx_config(
        &self,
        context: &WebBackendRequestContext,
        config_id: &str,
    ) -> WebServiceResult<NginxConfigResponse>;

    async fn update_nginx_config(
        &self,
        context: &WebBackendRequestContext,
        config_id: &str,
        request: &UpdateNginxConfigRequest,
    ) -> WebServiceResult<NginxConfigResponse>;

    async fn validate_nginx_config(
        &self,
        context: &WebBackendRequestContext,
        config_id: &str,
    ) -> WebServiceResult<NginxValidateResponse>;

    async fn web_nginx_config(
        &self,
        context: &WebBackendRequestContext,
        config_id: &str,
    ) -> WebServiceResult<NginxConfigResponse>;

    async fn reload_nginx(
        &self,
        context: &WebBackendRequestContext,
    ) -> WebServiceResult<NginxReloadResponse>;

    async fn retrieve_nginx_status(
        &self,
        context: &WebBackendRequestContext,
    ) -> WebServiceResult<NginxStatusResponse>;

    async fn list_servers(
        &self,
        context: &WebBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<ServerPage>;

    async fn create_server(
        &self,
        context: &WebBackendRequestContext,
        request: &CreateServerRequest,
    ) -> WebServiceResult<CreateServerResponse>;

    async fn list_audit_logs(
        &self,
        context: &WebBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<AuditLogPage>;
}
