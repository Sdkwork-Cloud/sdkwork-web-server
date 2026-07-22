use std::sync::Arc;

use async_trait::async_trait;
use sdkwork_webserver_core::website_runtime::WebsiteRuntimeRegistry;

use super::{WebsiteProviderEvent, WebsiteProviderEventReconciler};
use crate::WebsiteProviderRegistry;

pub struct WebsiteRuntimeSetProviderEventReconciler {
    runtime: Arc<WebsiteRuntimeRegistry>,
    providers: Arc<WebsiteProviderRegistry>,
    validation_concurrency: usize,
}

impl WebsiteRuntimeSetProviderEventReconciler {
    pub fn new(
        runtime: Arc<WebsiteRuntimeRegistry>,
        providers: Arc<WebsiteProviderRegistry>,
        validation_concurrency: usize,
    ) -> Self {
        Self {
            runtime,
            providers,
            validation_concurrency,
        }
    }
}

#[async_trait]
impl WebsiteProviderEventReconciler for WebsiteRuntimeSetProviderEventReconciler {
    async fn reconcile(&self, _event: &WebsiteProviderEvent) -> Result<(), String> {
        let runtime = self
            .runtime
            .current()
            .ok_or_else(|| "website runtime-set is unavailable".to_owned())?;
        self.providers
            .validate_runtime_set(&runtime, self.validation_concurrency)
            .await
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}
