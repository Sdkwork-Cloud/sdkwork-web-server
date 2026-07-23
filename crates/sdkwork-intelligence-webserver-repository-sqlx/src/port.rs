// `WebRepositoryPort` implementation delegated to the engine-specific repository modules.

use async_trait::async_trait;
use sdkwork_intelligence_webserver_service::{
    AuditLogWrite, RuntimeAssignmentTarget, RuntimeAssignmentWrite, RuntimeObservationWrite,
    WebRepositoryPort,
};
use sdkwork_webserver_contract::{
    AgentHeartbeatRequest, AgentHeartbeatResponse, AgentSyncResponse, AuditLogPage,
    CertificateIssueUpdate, CertificatePage, CertificateResponse, CreateCertificateRequest,
    CreateDeploymentRequest, CreateDomainRequest, CreateEnvVariableRequest,
    CreateHealthCheckRequest, CreateNginxConfigRequest, CreateServerRequest, CreateServerResponse,
    CreateSiteRequest, DeploymentPage, DeploymentResponse, DomainPage, DomainResponse,
    DomainVerifyResponse, EnvVariablePage, EnvVariableResponse, HealthCheckPage,
    HealthCheckResponse, ListNginxConfigsQuery, ListSitesQuery, NginxConfigPage,
    NginxConfigResponse, NginxReloadResponse, NginxStatusResponse, NginxValidateResponse,
    RuntimeAssignment, RuntimeAssignmentDelivery, RuntimeObservation, ServerPage, SitePage,
    SiteResponse, UpdateNginxConfigRequest, UpdateSiteRequest,
};
use sdkwork_webserver_contract::{WebServiceError, WebServiceResult};

use super::agents::AuthenticatedAgent;
use super::WebRepository;

#[async_trait]
impl WebRepositoryPort for WebRepository {
    async fn ready_check(&self) -> WebServiceResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|_| WebServiceError::DatabaseUnavailable)?;
        Ok(())
    }

    async fn list_sites(
        &self,
        tenant_id: i64,
        query: &ListSitesQuery,
    ) -> WebServiceResult<SitePage> {
        self.list_sites_repo(tenant_id, query).await
    }

    async fn create_site(
        &self,
        tenant_id: i64,
        organization_id: Option<i64>,
        actor_id: Option<i64>,
        request: &CreateSiteRequest,
    ) -> WebServiceResult<SiteResponse> {
        self.create_site_repo(tenant_id, organization_id, actor_id, request)
            .await
    }

    async fn retrieve_site(&self, tenant_id: i64, site_id: &str) -> WebServiceResult<SiteResponse> {
        self.retrieve_site_repo(tenant_id, site_id).await
    }

    async fn update_site(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &UpdateSiteRequest,
    ) -> WebServiceResult<SiteResponse> {
        self.update_site_repo(tenant_id, site_id, request).await
    }

    async fn delete_site(
        &self,
        tenant_id: i64,
        site_id: &str,
        actor_id: Option<i64>,
    ) -> WebServiceResult<()> {
        self.delete_site_repo(tenant_id, site_id, actor_id).await
    }

    async fn set_site_status(
        &self,
        tenant_id: i64,
        site_id: &str,
        status: i32,
    ) -> WebServiceResult<SiteResponse> {
        self.set_site_status_repo(tenant_id, site_id, status).await
    }

    async fn list_domains(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<DomainPage> {
        self.list_domains_repo(tenant_id, site_id, page, page_size)
            .await
    }

    async fn create_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateDomainRequest,
    ) -> WebServiceResult<DomainResponse> {
        self.create_domain_repo(tenant_id, site_id, request).await
    }

    async fn retrieve_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> WebServiceResult<DomainResponse> {
        self.retrieve_domain_repo(tenant_id, site_id, domain_id)
            .await
    }

    async fn delete_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> WebServiceResult<()> {
        self.delete_domain_repo(tenant_id, site_id, domain_id).await
    }

    async fn verify_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> WebServiceResult<DomainVerifyResponse> {
        self.verify_domain_repo(tenant_id, site_id, domain_id).await
    }

    async fn list_deployments(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
        status: Option<i32>,
    ) -> WebServiceResult<DeploymentPage> {
        self.list_deployments_repo(tenant_id, site_id, page, page_size, status)
            .await
    }

    async fn create_deployment(
        &self,
        tenant_id: i64,
        site_id: &str,
        actor_id: Option<i64>,
        request: &CreateDeploymentRequest,
    ) -> WebServiceResult<DeploymentResponse> {
        self.create_deployment_repo(tenant_id, site_id, actor_id, request)
            .await
    }

    async fn retrieve_deployment(
        &self,
        tenant_id: i64,
        site_id: &str,
        deployment_id: &str,
    ) -> WebServiceResult<DeploymentResponse> {
        self.retrieve_deployment_repo(tenant_id, site_id, deployment_id)
            .await
    }

    async fn rollback_deployment(
        &self,
        tenant_id: i64,
        site_id: &str,
        deployment_id: &str,
        actor_id: Option<i64>,
    ) -> WebServiceResult<DeploymentResponse> {
        self.rollback_deployment_repo(tenant_id, site_id, deployment_id, actor_id)
            .await
    }

    async fn list_env_variables(
        &self,
        tenant_id: i64,
        site_id: &str,
        environment: Option<&str>,
    ) -> WebServiceResult<EnvVariablePage> {
        self.list_env_variables_repo(tenant_id, site_id, environment)
            .await
    }

    async fn create_env_variable(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateEnvVariableRequest,
    ) -> WebServiceResult<EnvVariableResponse> {
        self.create_env_variable_repo(tenant_id, site_id, request)
            .await
    }

    async fn list_certificates(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<CertificatePage> {
        self.list_certificates_repo(tenant_id, page, page_size)
            .await
    }

    async fn create_certificate(
        &self,
        tenant_id: i64,
        request: &CreateCertificateRequest,
    ) -> WebServiceResult<CertificateResponse> {
        self.create_certificate_repo(tenant_id, request).await
    }

    async fn insert_certificate_pending(
        &self,
        tenant_id: i64,
        domain_id: &str,
        cert_type: i32,
        auto_renew: bool,
    ) -> WebServiceResult<(String, String)> {
        self.insert_certificate_pending_repo(tenant_id, domain_id, cert_type, auto_renew)
            .await
    }

    async fn finalize_certificate(
        &self,
        tenant_id: i64,
        certificate_id: &str,
        update: &CertificateIssueUpdate,
    ) -> WebServiceResult<CertificateResponse> {
        self.finalize_certificate_repo(tenant_id, certificate_id, update)
            .await
    }

    async fn fail_certificate(
        &self,
        tenant_id: i64,
        certificate_id: &str,
        reason: &str,
    ) -> WebServiceResult<()> {
        self.fail_certificate_repo(tenant_id, certificate_id, reason)
            .await
    }

    async fn list_certificates_due_for_renewal(
        &self,
        renew_before_days: u32,
        limit: i32,
    ) -> WebServiceResult<Vec<sdkwork_webserver_contract::CertificateRenewalCandidate>> {
        self.list_certificates_due_for_renewal_repo(renew_before_days, limit)
            .await
    }

    async fn mark_certificate_renewing(
        &self,
        tenant_id: i64,
        certificate_id: &str,
    ) -> WebServiceResult<bool> {
        self.mark_certificate_renewing_repo(tenant_id, certificate_id)
            .await
    }

    async fn fail_certificate_renewal(
        &self,
        tenant_id: i64,
        certificate_id: &str,
        reason: &str,
    ) -> WebServiceResult<()> {
        self.fail_certificate_renewal_repo(tenant_id, certificate_id, reason)
            .await
    }

    async fn list_health_checks(
        &self,
        tenant_id: i64,
        site_id: &str,
    ) -> WebServiceResult<HealthCheckPage> {
        self.list_health_checks_repo(tenant_id, site_id).await
    }

    async fn create_health_check(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateHealthCheckRequest,
    ) -> WebServiceResult<HealthCheckResponse> {
        self.create_health_check_repo(tenant_id, site_id, request)
            .await
    }

    async fn list_nginx_configs(
        &self,
        tenant_id: Option<i64>,
        query: &ListNginxConfigsQuery,
    ) -> WebServiceResult<NginxConfigPage> {
        self.list_nginx_configs_repo(tenant_id, query).await
    }

    async fn create_nginx_config(
        &self,
        tenant_id: i64,
        request: &CreateNginxConfigRequest,
    ) -> WebServiceResult<NginxConfigResponse> {
        self.create_nginx_config_repo(tenant_id, request).await
    }

    async fn retrieve_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> WebServiceResult<NginxConfigResponse> {
        self.retrieve_nginx_config_repo(tenant_id, config_id).await
    }

    async fn update_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
        request: &UpdateNginxConfigRequest,
    ) -> WebServiceResult<NginxConfigResponse> {
        self.update_nginx_config_repo(tenant_id, config_id, request)
            .await
    }

    async fn validate_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> WebServiceResult<NginxValidateResponse> {
        self.validate_nginx_config_repo(tenant_id, config_id).await
    }

    async fn load_nginx_config_content(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> WebServiceResult<String> {
        self.load_nginx_config_content_repo(tenant_id, config_id)
            .await
    }

    async fn resolve_site_primary_hostname(
        &self,
        tenant_id: i64,
        site_uuid: &str,
    ) -> WebServiceResult<String> {
        self.resolve_site_primary_hostname_repo(tenant_id, site_uuid)
            .await
    }

    async fn web_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> WebServiceResult<NginxConfigResponse> {
        self.web_nginx_config_repo(tenant_id, config_id).await
    }

    async fn reload_nginx(&self) -> WebServiceResult<NginxReloadResponse> {
        self.reload_nginx_repo().await
    }

    async fn retrieve_nginx_status(
        &self,
        tenant_id: Option<i64>,
    ) -> WebServiceResult<NginxStatusResponse> {
        self.retrieve_nginx_status_repo(tenant_id).await
    }

    async fn list_servers(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<ServerPage> {
        self.list_servers_repo(tenant_id, page, page_size).await
    }

    async fn create_server(
        &self,
        tenant_id: i64,
        request: &CreateServerRequest,
    ) -> WebServiceResult<CreateServerResponse> {
        self.create_server_repo(tenant_id, request).await
    }

    async fn authenticate_agent_token(&self, token: &str) -> WebServiceResult<(String, i64)> {
        let agent = self.authenticate_agent_token_repo(token).await?;
        Ok((agent.server_uuid, agent.tenant_id))
    }

    async fn resolve_runtime_assignment_target(
        &self,
        requester_tenant_id: i64,
        can_cross_tenant: bool,
        node_uuid: &str,
    ) -> WebServiceResult<RuntimeAssignmentTarget> {
        self.resolve_runtime_assignment_target_repo(
            requester_tenant_id,
            can_cross_tenant,
            node_uuid,
        )
        .await
    }

    async fn publish_runtime_assignment(
        &self,
        write: RuntimeAssignmentWrite,
    ) -> WebServiceResult<RuntimeAssignment> {
        self.publish_runtime_assignment_repo(write).await
    }

    async fn retrieve_current_runtime_assignment(
        &self,
        tenant_id: i64,
        node_uuid: &str,
        environment: &str,
        if_generation: Option<&str>,
        if_snapshot_sha256: Option<&str>,
    ) -> WebServiceResult<RuntimeAssignmentDelivery> {
        self.retrieve_current_runtime_assignment_repo(
            tenant_id,
            node_uuid,
            environment,
            if_generation,
            if_snapshot_sha256,
        )
        .await
    }

    async fn create_runtime_observation(
        &self,
        write: RuntimeObservationWrite,
    ) -> WebServiceResult<RuntimeObservation> {
        self.create_runtime_observation_repo(write).await
    }

    async fn retrieve_latest_runtime_observation(
        &self,
        requester_tenant_id: i64,
        can_cross_tenant: bool,
        snapshot_uuid: &str,
    ) -> WebServiceResult<RuntimeObservation> {
        self.retrieve_latest_runtime_observation_repo(
            requester_tenant_id,
            can_cross_tenant,
            snapshot_uuid,
        )
        .await
    }

    async fn record_agent_heartbeat(
        &self,
        server_id: &str,
        tenant_id: i64,
        request: &AgentHeartbeatRequest,
    ) -> WebServiceResult<AgentHeartbeatResponse> {
        let agent = AuthenticatedAgent {
            server_uuid: server_id.to_string(),
            tenant_id,
        };
        self.record_agent_heartbeat_repo(&agent, request).await
    }

    async fn build_agent_sync_manifest(
        &self,
        server_id: &str,
        tenant_id: i64,
        if_sync_version: Option<&str>,
    ) -> WebServiceResult<(AgentSyncResponse, Vec<String>)> {
        let agent = AuthenticatedAgent {
            server_uuid: server_id.to_string(),
            tenant_id,
        };
        self.build_agent_sync_manifest_repo(&agent, if_sync_version)
            .await
    }

    async fn list_audit_logs(
        &self,
        tenant_id: Option<i64>,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<AuditLogPage> {
        self.list_audit_logs_repo(tenant_id, page, page_size).await
    }

    async fn insert_audit_log(&self, entry: AuditLogWrite<'_>) -> WebServiceResult<()> {
        self.insert_audit_log_repo(entry).await
    }
}
