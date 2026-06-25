//! Repository port consumed by the Web service layer.

use async_trait::async_trait;
use sdkwork_webserver_contract::WebServiceResult;
use sdkwork_webserver_contract::{
    AgentHeartbeatRequest, AgentHeartbeatResponse, AgentSyncResponse, AuditLogPage,
    CertificateIssueUpdate, CertificatePage, CertificateResponse, CreateCertificateRequest,
    CreateDeploymentRequest, CreateDomainRequest, CreateEnvVariableRequest,
    CreateHealthCheckRequest, CreateNginxConfigRequest, CreateServerRequest, CreateServerResponse,
    CreateSiteRequest, DeploymentPage, DeploymentResponse, DomainPage, DomainResponse,
    DomainVerifyResponse, EnvVariablePage, EnvVariableResponse, HealthCheckPage,
    HealthCheckResponse, ListNginxConfigsQuery, ListSitesQuery, NginxConfigPage,
    NginxConfigResponse, NginxReloadResponse, NginxStatusResponse, NginxValidateResponse,
    ServerPage, SitePage, SiteResponse, UpdateNginxConfigRequest, UpdateSiteRequest,
};

#[async_trait]
pub trait WebRepositoryPort: Send + Sync {
    async fn ready_check(&self) -> WebServiceResult<()>;

    async fn list_sites(
        &self,
        tenant_id: i64,
        query: &ListSitesQuery,
    ) -> WebServiceResult<SitePage>;

    async fn create_site(
        &self,
        tenant_id: i64,
        organization_id: Option<i64>,
        actor_id: Option<i64>,
        request: &CreateSiteRequest,
    ) -> WebServiceResult<SiteResponse>;

    async fn retrieve_site(&self, tenant_id: i64, site_id: &str) -> WebServiceResult<SiteResponse>;

    async fn update_site(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &UpdateSiteRequest,
    ) -> WebServiceResult<SiteResponse>;

    async fn delete_site(
        &self,
        tenant_id: i64,
        site_id: &str,
        actor_id: Option<i64>,
    ) -> WebServiceResult<()>;

    async fn set_site_status(
        &self,
        tenant_id: i64,
        site_id: &str,
        status: i32,
    ) -> WebServiceResult<SiteResponse>;

    async fn list_domains(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<DomainPage>;

    async fn create_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateDomainRequest,
    ) -> WebServiceResult<DomainResponse>;

    async fn retrieve_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> WebServiceResult<DomainResponse>;

    async fn delete_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> WebServiceResult<()>;

    async fn verify_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> WebServiceResult<DomainVerifyResponse>;

    async fn list_deployments(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
        status: Option<i32>,
    ) -> WebServiceResult<DeploymentPage>;

    async fn create_deployment(
        &self,
        tenant_id: i64,
        site_id: &str,
        actor_id: Option<i64>,
        request: &CreateDeploymentRequest,
    ) -> WebServiceResult<DeploymentResponse>;

    async fn retrieve_deployment(
        &self,
        tenant_id: i64,
        site_id: &str,
        deployment_id: &str,
    ) -> WebServiceResult<DeploymentResponse>;

    async fn rollback_deployment(
        &self,
        tenant_id: i64,
        site_id: &str,
        deployment_id: &str,
        actor_id: Option<i64>,
    ) -> WebServiceResult<DeploymentResponse>;

    async fn list_env_variables(
        &self,
        tenant_id: i64,
        site_id: &str,
        environment: Option<&str>,
    ) -> WebServiceResult<EnvVariablePage>;

    async fn create_env_variable(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateEnvVariableRequest,
    ) -> WebServiceResult<EnvVariableResponse>;

    async fn list_certificates(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<CertificatePage>;

    async fn create_certificate(
        &self,
        tenant_id: i64,
        request: &CreateCertificateRequest,
    ) -> WebServiceResult<CertificateResponse>;

    async fn insert_certificate_pending(
        &self,
        tenant_id: i64,
        domain_id: &str,
        cert_type: i32,
        auto_renew: bool,
    ) -> WebServiceResult<(String, String)>;

    async fn finalize_certificate(
        &self,
        tenant_id: i64,
        certificate_id: &str,
        update: &CertificateIssueUpdate,
    ) -> WebServiceResult<CertificateResponse>;

    async fn fail_certificate(
        &self,
        tenant_id: i64,
        certificate_id: &str,
        reason: &str,
    ) -> WebServiceResult<()>;

    async fn list_certificates_due_for_renewal(
        &self,
        renew_before_days: u32,
        limit: i32,
    ) -> WebServiceResult<Vec<sdkwork_webserver_contract::CertificateRenewalCandidate>>;

    async fn mark_certificate_renewing(
        &self,
        tenant_id: i64,
        certificate_id: &str,
    ) -> WebServiceResult<bool>;

    async fn fail_certificate_renewal(
        &self,
        tenant_id: i64,
        certificate_id: &str,
        reason: &str,
    ) -> WebServiceResult<()>;

    async fn list_health_checks(
        &self,
        tenant_id: i64,
        site_id: &str,
    ) -> WebServiceResult<HealthCheckPage>;

    async fn create_health_check(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateHealthCheckRequest,
    ) -> WebServiceResult<HealthCheckResponse>;

    async fn list_nginx_configs(
        &self,
        tenant_id: Option<i64>,
        query: &ListNginxConfigsQuery,
    ) -> WebServiceResult<NginxConfigPage>;

    async fn create_nginx_config(
        &self,
        tenant_id: i64,
        request: &CreateNginxConfigRequest,
    ) -> WebServiceResult<NginxConfigResponse>;

    async fn retrieve_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> WebServiceResult<NginxConfigResponse>;

    async fn update_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
        request: &UpdateNginxConfigRequest,
    ) -> WebServiceResult<NginxConfigResponse>;

    async fn validate_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> WebServiceResult<NginxValidateResponse>;

    async fn load_nginx_config_content(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> WebServiceResult<String>;

    async fn resolve_site_primary_hostname(
        &self,
        tenant_id: i64,
        site_uuid: &str,
    ) -> WebServiceResult<String>;

    async fn web_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> WebServiceResult<NginxConfigResponse>;

    async fn reload_nginx(&self) -> WebServiceResult<NginxReloadResponse>;

    async fn retrieve_nginx_status(
        &self,
        tenant_id: Option<i64>,
    ) -> WebServiceResult<NginxStatusResponse>;

    async fn list_servers(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<ServerPage>;

    async fn create_server(
        &self,
        tenant_id: i64,
        request: &CreateServerRequest,
    ) -> WebServiceResult<CreateServerResponse>;

    async fn authenticate_agent_token(&self, token: &str) -> WebServiceResult<(String, i64)>;

    async fn record_agent_heartbeat(
        &self,
        server_id: &str,
        tenant_id: i64,
        request: &AgentHeartbeatRequest,
    ) -> WebServiceResult<AgentHeartbeatResponse>;

    async fn build_agent_sync_manifest(
        &self,
        server_id: &str,
        tenant_id: i64,
        if_sync_version: Option<&str>,
    ) -> WebServiceResult<(AgentSyncResponse, Vec<String>)>;

    async fn list_audit_logs(
        &self,
        tenant_id: Option<i64>,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<AuditLogPage>;

    async fn insert_audit_log(
        &self,
        tenant_id: i64,
        organization_id: i64,
        operator_id: i64,
        action: &str,
        target_type: &str,
        target_id: Option<i64>,
        target_uuid: Option<&str>,
    ) -> WebServiceResult<()>;
}
