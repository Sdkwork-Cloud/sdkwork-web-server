//! App-api service surface implementation.

use async_trait::async_trait;
use sdkwork_webserver_contract::{
    CreateCertificateRequest, CreateDeploymentRequest, CreateDomainRequest,
    CreateEnvVariableRequest, CreateHealthCheckRequest, CreateSiteRequest, ListSitesQuery,
    UpdateSiteRequest, WebAppApi, WebAppRequestContext, WebServiceResult,
};

use crate::{AuditLogWrite, WebService};

impl WebService {
    fn require_tenant(context: &WebAppRequestContext) -> WebServiceResult<i64> {
        if context.tenant_id <= 0 {
            return Err(sdkwork_webserver_contract::WebServiceError::Forbidden);
        }
        Ok(context.tenant_id)
    }

    async fn audit_site_action(
        &self,
        context: &WebAppRequestContext,
        action: &str,
        target_uuid: &str,
    ) -> WebServiceResult<()> {
        let operator_id = context.actor_id.unwrap_or(0);
        self.repository
            .insert_audit_log(AuditLogWrite {
                tenant_id: context.tenant_id,
                organization_id: context.organization_id.unwrap_or(0),
                operator_id,
                action,
                target_type: "site",
                target_id: None,
                target_uuid: Some(target_uuid),
            })
            .await
    }
}

#[async_trait]
impl WebAppApi for WebService {
    async fn list_sites(
        &self,
        context: &WebAppRequestContext,
        query: &ListSitesQuery,
    ) -> WebServiceResult<sdkwork_webserver_contract::SitePage> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository.list_sites(tenant_id, query).await
    }

    async fn create_site(
        &self,
        context: &WebAppRequestContext,
        request: &CreateSiteRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::SiteResponse> {
        let tenant_id = Self::require_tenant(context)?;
        let site = self
            .repository
            .create_site(
                tenant_id,
                context.organization_id,
                context.actor_id,
                request,
            )
            .await?;
        let _ = self
            .audit_site_action(context, "sites.create", &site.id)
            .await;
        Ok(site)
    }

    async fn retrieve_site(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::SiteResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository.retrieve_site(tenant_id, site_id).await
    }

    async fn update_site(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        request: &UpdateSiteRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::SiteResponse> {
        let tenant_id = Self::require_tenant(context)?;
        let site = self
            .repository
            .update_site(tenant_id, site_id, request)
            .await?;
        let _ = self
            .audit_site_action(context, "sites.update", site_id)
            .await;
        Ok(site)
    }

    async fn delete_site(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
    ) -> WebServiceResult<()> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .delete_site(tenant_id, site_id, context.actor_id)
            .await?;
        let _ = self
            .audit_site_action(context, "sites.delete", site_id)
            .await;
        Ok(())
    }

    async fn activate_site(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::SiteResponse> {
        let tenant_id = Self::require_tenant(context)?;
        let site = self
            .repository
            .set_site_status(tenant_id, site_id, 1)
            .await?;
        let _ = self
            .audit_site_action(context, "sites.activate", site_id)
            .await;
        Ok(site)
    }

    async fn pause_site(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::SiteResponse> {
        let tenant_id = Self::require_tenant(context)?;
        let site = self
            .repository
            .set_site_status(tenant_id, site_id, 2)
            .await?;
        let _ = self
            .audit_site_action(context, "sites.pause", site_id)
            .await;
        Ok(site)
    }

    async fn list_domains(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<sdkwork_webserver_contract::DomainPage> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .list_domains(tenant_id, site_id, page, page_size)
            .await
    }

    async fn create_domain(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        request: &CreateDomainRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::DomainResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .create_domain(tenant_id, site_id, request)
            .await
    }

    async fn retrieve_domain(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        domain_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::DomainResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .retrieve_domain(tenant_id, site_id, domain_id)
            .await
    }

    async fn delete_domain(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        domain_id: &str,
    ) -> WebServiceResult<()> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .delete_domain(tenant_id, site_id, domain_id)
            .await
    }

    async fn verify_domain(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        domain_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::DomainVerifyResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .verify_domain(tenant_id, site_id, domain_id)
            .await
    }

    async fn list_deployments(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        page: i32,
        page_size: i32,
        status: Option<i32>,
    ) -> WebServiceResult<sdkwork_webserver_contract::DeploymentPage> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .list_deployments(tenant_id, site_id, page, page_size, status)
            .await
    }

    async fn create_deployment(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        request: &CreateDeploymentRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::DeploymentResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .create_deployment(tenant_id, site_id, context.actor_id, request)
            .await
    }

    async fn retrieve_deployment(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        deployment_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::DeploymentResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .retrieve_deployment(tenant_id, site_id, deployment_id)
            .await
    }

    async fn rollback_deployment(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        deployment_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::DeploymentResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .rollback_deployment(tenant_id, site_id, deployment_id, context.actor_id)
            .await
    }

    async fn list_env_variables(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        environment: Option<&str>,
    ) -> WebServiceResult<sdkwork_webserver_contract::EnvVariablePage> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .list_env_variables(tenant_id, site_id, environment)
            .await
    }

    async fn create_env_variable(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        request: &CreateEnvVariableRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::EnvVariableResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .create_env_variable(tenant_id, site_id, request)
            .await
    }

    async fn list_certificates(
        &self,
        context: &WebAppRequestContext,
        page: i32,
        page_size: i32,
    ) -> WebServiceResult<sdkwork_webserver_contract::CertificatePage> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .list_certificates(tenant_id, page, page_size)
            .await
    }

    async fn create_certificate(
        &self,
        context: &WebAppRequestContext,
        request: &CreateCertificateRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::CertificateResponse> {
        self.issue_certificate(context, request).await
    }

    async fn list_health_checks(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::HealthCheckPage> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository.list_health_checks(tenant_id, site_id).await
    }

    async fn create_health_check(
        &self,
        context: &WebAppRequestContext,
        site_id: &str,
        request: &CreateHealthCheckRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::HealthCheckResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .create_health_check(tenant_id, site_id, request)
            .await
    }
}
