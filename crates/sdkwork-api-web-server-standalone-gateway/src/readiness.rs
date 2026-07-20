use std::sync::Arc;

use sdkwork_intelligence_webserver_service::WebService;
use sdkwork_web_bootstrap::{ReadinessCheck, ReadinessFuture};

pub struct WebServiceReadinessCheck {
    service: Arc<WebService>,
}

impl WebServiceReadinessCheck {
    pub fn new(service: Arc<WebService>) -> Self {
        Self { service }
    }
}

impl ReadinessCheck for WebServiceReadinessCheck {
    fn check(&self) -> ReadinessFuture<'_> {
        let service = self.service.clone();
        Box::pin(async move {
            service
                .ready_check()
                .await
                .map_err(|error| error.to_string())
        })
    }
}
