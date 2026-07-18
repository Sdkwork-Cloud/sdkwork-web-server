use std::sync::Arc;

use axum::Router;
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_routes_webserver_common::{
    web_auth_mode_from_env, with_problem_correlation, ProductionFailClosedResolver, WebAuthMode,
};
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{
    DefaultWebRequestContextResolver, DomainContextInjector, HttpMetricsRegistry,
    WebRequestContext, WebRequestContextProfile,
};
use sdkwork_webserver_contract::WebAppRequestContext;

use crate::http_route_manifest::app_route_manifest;
use crate::paths;

pub fn web_app_api_public_path_prefixes() -> Vec<String> {
    Vec::new()
}

pub fn web_app_api_prefixes() -> Vec<String> {
    vec![paths::PREFIX.to_owned()]
}

#[derive(Clone, Default)]
struct WebAppContextInjector;

impl DomainContextInjector for WebAppContextInjector {
    fn inject(&self, request: &mut axum::extract::Request, context: &WebRequestContext) {
        if let Some(app_context) = web_app_context_from_web_request(context) {
            request.extensions_mut().insert(app_context);
        }
    }
}

fn web_app_context_from_web_request(context: &WebRequestContext) -> Option<WebAppRequestContext> {
    let principal = context.principal.as_ref()?;
    let tenant_id = principal.tenant_id().parse().ok()?;
    let actor_id = principal.user_id().parse().ok();
    let organization_id = principal
        .organization_id()
        .and_then(|value| value.parse().ok());
    let session_id = principal.session_id().map(str::to_owned);
    Some(WebAppRequestContext {
        tenant_id,
        actor_id,
        organization_id,
        session_id,
    })
}

pub fn wrap_router_with_web_framework(
    resolver: DefaultWebRequestContextResolver,
    router: Router,
) -> Router {
    with_web_request_context(
        with_problem_correlation(router),
        build_web_app_api_framework_layer(resolver, None),
    )
}

pub fn wrap_router_with_web_framework_and_metrics(
    resolver: DefaultWebRequestContextResolver,
    router: Router,
    metrics: Arc<HttpMetricsRegistry>,
) -> Router {
    with_web_request_context(
        with_problem_correlation(router),
        build_web_app_api_framework_layer(resolver, Some(metrics)),
    )
}

pub fn wrap_router_with_iam_database_web_framework(
    resolver: IamWebRequestContextResolver,
    router: Router,
) -> Router {
    with_web_request_context(
        with_problem_correlation(router),
        build_web_app_api_framework_layer(resolver, None),
    )
}

fn build_web_app_api_framework_layer<R>(
    resolver: R,
    metrics: Option<Arc<HttpMetricsRegistry>>,
) -> WebFrameworkLayer<R>
where
    R: sdkwork_web_core::WebRequestContextResolver + Clone,
{
    let route_manifest = app_route_manifest();
    route_manifest
        .validate_public_path_prefixes(&web_app_api_public_path_prefixes())
        .expect("Web app-api public prefixes must not cover protected manifest routes");

    let layer = WebFrameworkLayer::new(resolver)
        .with_profile(WebRequestContextProfile {
            app_api_prefix: paths::PREFIX.to_owned(),
            public_path_prefixes: web_app_api_public_path_prefixes(),
            ..WebRequestContextProfile::default()
        })
        .with_route_manifest(route_manifest)
        .with_domain_injector(Arc::new(WebAppContextInjector));
    match metrics {
        Some(metrics) => layer.with_metrics(metrics),
        None => layer,
    }
}

pub async fn wrap_router_with_web_framework_from_env(router: Router) -> Router {
    wrap_router_with_web_framework_from_env_and_optional_metrics(router, None).await
}

pub async fn wrap_router_with_web_framework_from_env_and_metrics(
    router: Router,
    metrics: Arc<HttpMetricsRegistry>,
) -> Router {
    wrap_router_with_web_framework_from_env_and_optional_metrics(router, Some(metrics)).await
}

async fn wrap_router_with_web_framework_from_env_and_optional_metrics(
    router: Router,
    metrics: Option<Arc<HttpMetricsRegistry>>,
) -> Router {
    match web_auth_mode_from_env().await {
        WebAuthMode::DevInline => with_web_request_context(
            with_problem_correlation(router),
            build_web_app_api_framework_layer(DefaultWebRequestContextResolver::default(), metrics),
        ),
        WebAuthMode::ProductionFailClosed => with_web_request_context(
            with_problem_correlation(router),
            build_web_app_api_framework_layer(ProductionFailClosedResolver, metrics),
        ),
        WebAuthMode::IamDatabase(resolver) => with_web_request_context(
            with_problem_correlation(router),
            build_web_app_api_framework_layer(resolver, metrics),
        ),
    }
}
