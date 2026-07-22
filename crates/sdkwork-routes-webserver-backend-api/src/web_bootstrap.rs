use std::sync::Arc;

use axum::{Extension, Router};
use sdkwork_intelligence_webserver_service::WebService;
use sdkwork_routes_webserver_common::{
    web_auth_mode_from_env, with_problem_correlation, MachineCredentialResolverDecorator,
    ProductionFailClosedResolver, WebAuthMode,
};
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{
    DefaultWebRequestContextResolver, DomainContextInjector, HttpMetricsRegistry,
    WebRequestContext, WebRequestContextProfile, WebRequestContextResolver,
};
use sdkwork_webserver_contract::{MachineCredentialAuthenticator, WebBackendRequestContext};

use crate::http_route_manifest::backend_route_manifest;
use crate::paths;

#[derive(Clone, Default)]
struct WebBackendContextInjector;

impl DomainContextInjector for WebBackendContextInjector {
    fn inject(&self, request: &mut axum::extract::Request, context: &WebRequestContext) {
        if let Some(backend_context) = web_backend_context_from_web_request(context) {
            request.extensions_mut().insert(backend_context);
        }
    }
}

fn web_backend_context_from_web_request(
    context: &WebRequestContext,
) -> Option<WebBackendRequestContext> {
    let principal = context.principal.as_ref()?;
    let tenant_id: i64 = principal.tenant_id().parse().ok()?;
    // Machine principals use an opaque Web Node UUID, so an operator id is optional.
    let operator_id = principal.user_id().parse().ok();
    let subject_id = Some(principal.user_id().to_owned());
    Some(WebBackendRequestContext {
        operator_id,
        tenant_id: Some(tenant_id),
        subject_id,
    })
}

fn build_web_backend_api_framework_layer<R>(
    resolver: R,
    metrics: Option<Arc<HttpMetricsRegistry>>,
) -> WebFrameworkLayer<R>
where
    R: WebRequestContextResolver + Clone,
{
    let layer = WebFrameworkLayer::new(resolver)
        .with_profile(WebRequestContextProfile {
            backend_api_prefix: paths::PREFIX.to_owned(),
            ..WebRequestContextProfile::default()
        })
        .with_route_manifest(backend_route_manifest())
        .with_domain_injector(Arc::new(WebBackendContextInjector));
    match metrics {
        Some(metrics) => layer.with_metrics(metrics),
        None => layer,
    }
}

pub async fn wrap_router_with_web_framework_from_env(
    router: Router,
    service: Arc<WebService>,
) -> Router {
    wrap_router_with_web_framework_from_env_and_optional_metrics(router, service, None).await
}

pub async fn wrap_router_with_web_framework_from_env_and_metrics(
    router: Router,
    service: Arc<WebService>,
    metrics: Arc<HttpMetricsRegistry>,
) -> Router {
    wrap_router_with_web_framework_from_env_and_optional_metrics(router, service, Some(metrics))
        .await
}

async fn wrap_router_with_web_framework_from_env_and_optional_metrics(
    router: Router,
    service: Arc<WebService>,
    metrics: Option<Arc<HttpMetricsRegistry>>,
) -> Router {
    // Clone service for the resolver decorator before moving the original into Extension.
    let service_for_resolver = service.clone();
    let machine_authenticator: Arc<dyn MachineCredentialAuthenticator> = service_for_resolver;
    // Extension(service) is applied inside the framework layer so machine routes
    // can extract Arc<WebService> alongside the framework-injected WebBackendRequestContext.
    let correlated = with_problem_correlation(router).layer(Extension(service));
    match web_auth_mode_from_env().await {
        WebAuthMode::DevInline => with_web_request_context(
            correlated,
            build_web_backend_api_framework_layer(
                MachineCredentialResolverDecorator::new(
                    DefaultWebRequestContextResolver::default(),
                    machine_authenticator.clone(),
                ),
                metrics,
            ),
        ),
        WebAuthMode::ProductionFailClosed => with_web_request_context(
            correlated,
            build_web_backend_api_framework_layer(
                MachineCredentialResolverDecorator::new(
                    ProductionFailClosedResolver,
                    machine_authenticator.clone(),
                ),
                metrics,
            ),
        ),
        WebAuthMode::IamDatabase(resolver) => with_web_request_context(
            correlated,
            build_web_backend_api_framework_layer(
                MachineCredentialResolverDecorator::new(resolver, machine_authenticator),
                metrics,
            ),
        ),
    }
}
