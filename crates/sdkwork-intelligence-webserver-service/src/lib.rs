//! Web business service orchestrating repository ports and HTTP API traits.

pub mod agent_ops;
pub mod app;
pub mod backend;
pub mod certificate_ops;
pub mod certificate_renewal_ops;
pub mod nginx_ops;
pub mod repository;

pub use repository::WebRepositoryPort;

use std::sync::Arc;

use sdkwork_webserver_acme_service::CertificateIssuer;
use sdkwork_webserver_contract::WebServiceResult;
use sdkwork_webserver_edge_runtime::EdgeRuntime;

/// Application service for SDKWork Web control plane operations.
pub struct WebService {
    pub(crate) repository: Arc<dyn WebRepositoryPort>,
    pub(crate) certificate_issuer: Arc<CertificateIssuer>,
    pub(crate) edge_runtime: Arc<EdgeRuntime>,
}

impl WebService {
    pub fn new(
        repository: Arc<dyn WebRepositoryPort>,
        certificate_issuer: Arc<CertificateIssuer>,
        edge_runtime: Arc<EdgeRuntime>,
    ) -> Self {
        Self {
            repository,
            certificate_issuer,
            edge_runtime,
        }
    }

    pub async fn ready_check(&self) -> WebServiceResult<()> {
        self.repository.ready_check().await
    }
}
