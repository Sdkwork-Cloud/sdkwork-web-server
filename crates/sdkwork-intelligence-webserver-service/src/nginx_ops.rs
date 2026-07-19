//! Nginx deploy, validate, and reload orchestration through the edge runtime.

use sdkwork_webserver_contract::{WebServiceError, WebServiceResult};

use crate::WebService;

impl WebService {
    pub async fn validate_nginx_content(&self, content: &str) -> WebServiceResult<()> {
        let runtime = self.edge_runtime.clone();
        let content = content.to_owned();
        tokio::task::spawn_blocking(move || runtime.validate_config_content(&content))
            .await
            .map_err(|error| WebServiceError::Internal(format!("join nginx validation: {error}")))?
            .map_err(|error| WebServiceError::validation(error.to_string()))
    }

    pub async fn deploy_nginx_site(&self, domain: &str, content: &str) -> WebServiceResult<()> {
        let runtime = self.edge_runtime.clone();
        let domain = domain.to_owned();
        let content = content.to_owned();
        tokio::task::spawn_blocking(move || runtime.deploy_site_config(&domain, &content))
            .await
            .map_err(|error| WebServiceError::Internal(format!("join nginx deployment: {error}")))?
            .map_err(|error| WebServiceError::Internal(error.to_string()))
    }

    pub async fn reload_nginx_runtime(&self) -> WebServiceResult<()> {
        let runtime = self.edge_runtime.clone();
        tokio::task::spawn_blocking(move || runtime.reload())
            .await
            .map_err(|error| WebServiceError::Internal(format!("join nginx reload: {error}")))?
            .map_err(|error| WebServiceError::Internal(error.to_string()))
    }
}
