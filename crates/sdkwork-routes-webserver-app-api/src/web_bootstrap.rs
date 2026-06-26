use std::sync::Arc;

use axum::Router;
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_routes_webserver_common::{
    web_auth_mode_from_env, with_problem_correlation, ProductionFailClosedResolver, WebAuthMode,
};
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{
    DefaultWebRequestContextResolver, DomainContextInjector, WebRequestContext,
    WebRequestContextProfile,
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
        build_web_app_api_framework_layer(resolver),
    )
}

pub fn wrap_router_with_iam_database_web_framework(
    resolver: IamWebRequestContextResolver,
    router: Router,
) -> Router {
    with_web_request_context(
        with_problem_correlation(router),
        build_web_app_api_framework_layer(resolver),
    )
}

fn build_web_app_api_framework_layer<R>(resolver: R) -> WebFrameworkLayer<R>
where
    R: sdkwork_web_core::WebRequestContextResolver + Clone,
{
    let route_manifest = app_route_manifest();
    route_manifest
        .validate_public_path_prefixes(&web_app_api_public_path_prefixes())
        .expect("Web app-api public prefixes must not cover protected manifest routes");

    WebFrameworkLayer::new(resolver)
        .with_profile(WebRequestContextProfile {
            app_api_prefix: paths::PREFIX.to_owned(),
            public_path_prefixes: web_app_api_public_path_prefixes(),
            ..WebRequestContextProfile::default()
        })
        .with_route_manifest(route_manifest)
        .with_domain_injector(Arc::new(WebAppContextInjector))
}

pub async fn wrap_router_with_web_framework_from_env(router: Router) -> Router {
    match web_auth_mode_from_env().await {
        WebAuthMode::DevInline => {
            wrap_router_with_web_framework(DefaultWebRequestContextResolver::default(), router)
        }
        WebAuthMode::ProductionFailClosed => with_web_request_context(
            with_problem_correlation(router),
            build_web_app_api_framework_layer(ProductionFailClosedResolver),
        ),
        WebAuthMode::IamDatabase(resolver) => {
            wrap_router_with_iam_database_web_framework(resolver, router)
        }
    }
}
