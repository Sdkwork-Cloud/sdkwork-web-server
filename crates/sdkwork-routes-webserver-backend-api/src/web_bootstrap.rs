use std::sync::Arc;

use axum::Router;
use sdkwork_routes_webserver_common::{
    web_auth_mode_from_env, with_problem_correlation, ProductionFailClosedResolver, WebAuthMode,
};
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{
    DefaultWebRequestContextResolver, DomainContextInjector, WebRequestContext,
    WebRequestContextProfile,
};
use sdkwork_webserver_contract::WebBackendRequestContext;

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
    Some(WebBackendRequestContext {
        operator_id: principal.user_id().parse().ok(),
        tenant_id: principal.tenant_id().parse().ok(),
    })
}

fn build_web_backend_api_framework_layer<R>(resolver: R) -> WebFrameworkLayer<R>
where
    R: sdkwork_web_core::WebRequestContextResolver + Clone,
{
    WebFrameworkLayer::new(resolver)
        .with_profile(WebRequestContextProfile {
            backend_api_prefix: paths::PREFIX.to_owned(),
            ..WebRequestContextProfile::default()
        })
        .with_route_manifest(backend_route_manifest())
        .with_domain_injector(Arc::new(WebBackendContextInjector))
}

pub async fn wrap_router_with_web_framework_from_env(router: Router) -> Router {
    match web_auth_mode_from_env().await {
        WebAuthMode::DevInline => with_web_request_context(
            with_problem_correlation(router),
            build_web_backend_api_framework_layer(DefaultWebRequestContextResolver::default()),
        ),
        WebAuthMode::ProductionFailClosed => with_web_request_context(
            with_problem_correlation(router),
            build_web_backend_api_framework_layer(ProductionFailClosedResolver),
        ),
        WebAuthMode::IamDatabase(resolver) => with_web_request_context(
            with_problem_correlation(router),
            build_web_backend_api_framework_layer(resolver),
        ),
    }
}
