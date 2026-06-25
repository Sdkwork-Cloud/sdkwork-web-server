//! Shared Web router auth wiring for sdkwork-web-framework integration.

pub mod correlation;
pub mod problem;

use async_trait::async_trait;
use sdkwork_iam_web_adapter::IamDatabaseWebRequestContextResolver;
use sdkwork_web_core::{WebFrameworkError, WebRequestContextResolver, WebRequestPrincipal};
use sdkwork_webserver_contract::{
    web_is_production_like_environment, web_use_dev_inline_auth_resolver,
};

pub use correlation::{with_problem_correlation, WebProblemCorrelation};
pub use problem::{WebApiError, WebApiProblem, WebApiResult};

const PRODUCTION_AUTH_UNAVAILABLE: &str = "production Web auth requires IAM PostgreSQL database";

pub enum WebAuthMode {
    DevInline,
    IamDatabase(IamDatabaseWebRequestContextResolver),
    ProductionFailClosed,
}

pub async fn web_auth_mode_from_env() -> WebAuthMode {
    if web_use_dev_inline_auth_resolver() {
        return WebAuthMode::DevInline;
    }

    let iam_database_explicitly_configured = std::env::var("SDKWORK_IAM_DATABASE_URL")
        .or_else(|_| std::env::var("SDKWORK_IAM_DATABASE_ENGINE"))
        .is_ok();

    if web_is_production_like_environment() && !iam_database_explicitly_configured {
        return WebAuthMode::ProductionFailClosed;
    }

    WebAuthMode::IamDatabase(sdkwork_iam_web_adapter::iam_database_resolver_from_env().await)
}

#[derive(Clone, Default)]
pub struct ProductionFailClosedResolver;

#[async_trait]
impl WebRequestContextResolver for ProductionFailClosedResolver {
    async fn resolve_api_key(
        &self,
        _raw_api_key: &str,
    ) -> Result<WebRequestPrincipal, WebFrameworkError> {
        Err(WebFrameworkError::invalid_credentials(
            PRODUCTION_AUTH_UNAVAILABLE,
        ))
    }

    async fn resolve_dual_token(
        &self,
        _raw_auth_token: &str,
        _raw_access_token: &str,
    ) -> Result<WebRequestPrincipal, WebFrameworkError> {
        Err(WebFrameworkError::invalid_credentials(
            PRODUCTION_AUTH_UNAVAILABLE,
        ))
    }

    async fn resolve_access_token(
        &self,
        _raw_access_token: &str,
    ) -> Result<WebRequestPrincipal, WebFrameworkError> {
        Err(WebFrameworkError::invalid_credentials(
            PRODUCTION_AUTH_UNAVAILABLE,
        ))
    }

    async fn resolve_oauth_bearer(
        &self,
        _raw_bearer_token: &str,
    ) -> Result<WebRequestPrincipal, WebFrameworkError> {
        Err(WebFrameworkError::invalid_credentials(
            PRODUCTION_AUTH_UNAVAILABLE,
        ))
    }
}
