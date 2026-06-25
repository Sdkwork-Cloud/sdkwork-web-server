//! nginx deploy/validate/reload orchestration through edge runtime.

use sdkwork_webserver_contract::{WebServiceError, WebServiceResult};

use crate::WebService;

impl WebService {
    pub fn validate_nginx_content(&self, content: &str) -> WebServiceResult<()> {
        self.edge_runtime
            .validate_config_content(content)
            .map_err(|error| WebServiceError::validation(error.to_string()))
    }

    pub fn deploy_nginx_site(&self, domain: &str, content: &str) -> WebServiceResult<()> {
        self.edge_runtime
            .deploy_site_config(domain, content)
            .map_err(|error| WebServiceError::Internal(error.to_string()))
    }

    pub fn reload_nginx_runtime(&self) -> WebServiceResult<()> {
        self.edge_runtime
            .reload()
            .map_err(|error| WebServiceError::Internal(error.to_string()))
    }
}
