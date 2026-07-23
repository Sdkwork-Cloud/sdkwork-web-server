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
    RuntimeAssignment, RuntimeAssignmentDelivery, RuntimeObservation, RuntimeObservationState,
    ServerPage, SitePage, SiteResponse, UpdateNginxConfigRequest, UpdateSiteRequest,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAssignmentTarget {
    pub server_id: i64,
    pub node_uuid: String,
    pub tenant_id: i64,
    pub tenant_scope_hash: String,
}

#[derive(Clone, Debug)]
pub struct RuntimeAssignmentWrite {
    pub tenant_id: i64,
    pub server_id: i64,
    pub node_uuid: String,
    pub environment: String,
    pub generation: u64,
    pub snapshot_uuid: String,
    pub snapshot_sha256: String,
    pub runtime_set_json: String,
    pub runtime_set_bytes: usize,
    pub assigned_by_subject: String,
}

#[derive(Clone, Debug)]
pub struct RuntimeObservationWrite {
    pub tenant_id: i64,
    pub node_uuid: String,
    pub snapshot_uuid: String,
    pub generation: u64,
    pub snapshot_sha256: String,
    pub state: RuntimeObservationState,
    pub node_version: Option<String>,
    pub reason_code: Option<String>,
    pub detail: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub struct AuditLogWrite<'a> {
    pub tenant_id: i64,
    pub organization_id: i64,
    pub operator_id: i64,
    pub action: &'a str,
    pub target_type: &'a str,
    pub target_id: Option<i64>,
    pub target_uuid: Option<&'a str>,
}

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

    async fn resolve_runtime_assignment_target(
        &self,
        requester_tenant_id: i64,
        can_cross_tenant: bool,
        node_uuid: &str,
    ) -> WebServiceResult<RuntimeAssignmentTarget>;

    async fn publish_runtime_assignment(
        &self,
        write: RuntimeAssignmentWrite,
    ) -> WebServiceResult<RuntimeAssignment>;

    async fn retrieve_current_runtime_assignment(
        &self,
        tenant_id: i64,
        node_uuid: &str,
        environment: &str,
        if_generation: Option<&str>,
        if_snapshot_sha256: Option<&str>,
    ) -> WebServiceResult<RuntimeAssignmentDelivery>;

    async fn create_runtime_observation(
        &self,
        write: RuntimeObservationWrite,
    ) -> WebServiceResult<RuntimeObservation>;

    async fn retrieve_latest_runtime_observation(
        &self,
        requester_tenant_id: i64,
        can_cross_tenant: bool,
        snapshot_uuid: &str,
    ) -> WebServiceResult<RuntimeObservation>;

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

    async fn insert_audit_log(&self, entry: AuditLogWrite<'_>) -> WebServiceResult<()>;
}
