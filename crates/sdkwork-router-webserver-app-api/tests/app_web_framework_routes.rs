use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_iam_web_adapter::IamDatabaseWebRequestContextResolver;
use sdkwork_router_webserver_app_api::{
    build_router_with_shared_app_api, web_bootstrap::wrap_router_with_iam_database_web_framework,
};
use sdkwork_webserver_contract::{
    ListSitesQuery, SitePage, WebAppApi, WebAppRequestContext, WebServiceResult,
};
use std::sync::Arc;
use tower::util::ServiceExt;

#[tokio::test]
async fn app_router_web_framework_rejects_unauthenticated_requests() {
    let app = wrap_router_with_iam_database_web_framework(
        IamDatabaseWebRequestContextResolver::new(None),
        build_router_with_shared_app_api(Arc::new(StubAppApi)),
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/app/v3/api/sites")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

struct StubAppApi;

#[async_trait]
impl WebAppApi for StubAppApi {
    async fn list_sites(
        &self,
        _context: &WebAppRequestContext,
        _query: &ListSitesQuery,
    ) -> WebServiceResult<SitePage> {
        Ok(SitePage::default())
    }

    async fn create_site(
        &self,
        _context: &WebAppRequestContext,
        _request: &sdkwork_webserver_contract::CreateSiteRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::SiteResponse> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn retrieve_site(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::SiteResponse> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn update_site(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
        _request: &sdkwork_webserver_contract::UpdateSiteRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::SiteResponse> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn delete_site(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
    ) -> WebServiceResult<()> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn activate_site(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::SiteResponse> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn pause_site(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::SiteResponse> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn list_domains(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
        _page: i32,
        _page_size: i32,
    ) -> WebServiceResult<sdkwork_webserver_contract::DomainPage> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn create_domain(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
        _request: &sdkwork_webserver_contract::CreateDomainRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::DomainResponse> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn retrieve_domain(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
        _domain_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::DomainResponse> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn delete_domain(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
        _domain_id: &str,
    ) -> WebServiceResult<()> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn verify_domain(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
        _domain_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::DomainVerifyResponse> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn list_deployments(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
        _page: i32,
        _page_size: i32,
        _status: Option<i32>,
    ) -> WebServiceResult<sdkwork_webserver_contract::DeploymentPage> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn create_deployment(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
        _request: &sdkwork_webserver_contract::CreateDeploymentRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::DeploymentResponse> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn retrieve_deployment(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
        _deployment_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::DeploymentResponse> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn rollback_deployment(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
        _deployment_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::DeploymentResponse> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn list_env_variables(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
        _environment: Option<&str>,
    ) -> WebServiceResult<sdkwork_webserver_contract::EnvVariablePage> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn create_env_variable(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
        _request: &sdkwork_webserver_contract::CreateEnvVariableRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::EnvVariableResponse> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn list_certificates(
        &self,
        _context: &WebAppRequestContext,
        _page: i32,
        _page_size: i32,
    ) -> WebServiceResult<sdkwork_webserver_contract::CertificatePage> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn create_certificate(
        &self,
        _context: &WebAppRequestContext,
        _request: &sdkwork_webserver_contract::CreateCertificateRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::CertificateResponse> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn list_health_checks(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
    ) -> WebServiceResult<sdkwork_webserver_contract::HealthCheckPage> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn create_health_check(
        &self,
        _context: &WebAppRequestContext,
        _site_id: &str,
        _request: &sdkwork_webserver_contract::CreateHealthCheckRequest,
    ) -> WebServiceResult<sdkwork_webserver_contract::HealthCheckResponse> {
        Err(sdkwork_webserver_contract::WebServiceError::Internal(
            "not implemented".into(),
        ))
    }
}
