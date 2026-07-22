use std::sync::Arc;

use axum::Router;
use sdkwork_routes_webserver_common::{
    web_auth_mode_from_env, with_problem_correlation, MachineCredentialResolverDecorator,
    ProductionFailClosedResolver, WebAuthMode,
};
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{
    DefaultWebRequestContextResolver, DomainContextInjector, HttpMetricsRegistry,
    WebRequestContext, WebRequestContextProfile, WebRequestContextResolver,
};
use sdkwork_webserver_contract::{MachineCredentialAuthenticator, WebInternalRequestContext};

use crate::http_route_manifest::internal_route_manifest;

const WEB_AGENT_APP_ID: &str = "sdkwork-web-agent";
const RUNTIME_ASSIGNMENT_WRITE_PERMISSION: &str = "web.runtimeAssignments.write";

#[derive(Clone, Default)]
struct WebInternalContextInjector;

impl DomainContextInjector for WebInternalContextInjector {
    fn inject(&self, request: &mut axum::extract::Request, context: &WebRequestContext) {
        if let Some(internal_context) = web_internal_context_from_web_request(context) {
            request.extensions_mut().insert(internal_context);
        }
    }
}

fn web_internal_context_from_web_request(
    context: &WebRequestContext,
) -> Option<WebInternalRequestContext> {
    let principal = context.principal.as_ref()?;
    let tenant_id = principal.tenant_id().parse().ok()?;
    let is_web_agent = principal.app_id() == WEB_AGENT_APP_ID;
    Some(WebInternalRequestContext {
        tenant_id,
        subject_id: principal.user_id().to_owned(),
        agent_node_uuid: is_web_agent.then(|| principal.user_id().to_owned()),
        can_publish_cross_tenant: context.has_permission(RUNTIME_ASSIGNMENT_WRITE_PERMISSION),
    })
}

fn build_web_internal_api_framework_layer<R>(
    resolver: R,
    metrics: Option<Arc<HttpMetricsRegistry>>,
) -> WebFrameworkLayer<R>
where
    R: WebRequestContextResolver + Clone,
{
    let layer = WebFrameworkLayer::new(resolver)
        .with_profile(WebRequestContextProfile::default())
        .with_route_manifest(internal_route_manifest())
        .with_domain_injector(Arc::new(WebInternalContextInjector));
    match metrics {
        Some(metrics) => layer.with_metrics(metrics),
        None => layer,
    }
}

pub async fn wrap_router_with_web_framework_from_env(
    router: Router,
    machine_authenticator: Arc<dyn MachineCredentialAuthenticator>,
) -> Router {
    wrap_router_with_web_framework_from_env_and_optional_metrics(
        router,
        machine_authenticator,
        None,
    )
    .await
}

pub async fn wrap_router_with_web_framework_from_env_and_metrics(
    router: Router,
    machine_authenticator: Arc<dyn MachineCredentialAuthenticator>,
    metrics: Arc<HttpMetricsRegistry>,
) -> Router {
    wrap_router_with_web_framework_from_env_and_optional_metrics(
        router,
        machine_authenticator,
        Some(metrics),
    )
    .await
}

async fn wrap_router_with_web_framework_from_env_and_optional_metrics(
    router: Router,
    machine_authenticator: Arc<dyn MachineCredentialAuthenticator>,
    metrics: Option<Arc<HttpMetricsRegistry>>,
) -> Router {
    let correlated = with_problem_correlation(router);
    match web_auth_mode_from_env().await {
        WebAuthMode::DevInline => with_web_request_context(
            correlated,
            build_web_internal_api_framework_layer(
                MachineCredentialResolverDecorator::new(
                    DefaultWebRequestContextResolver::default(),
                    machine_authenticator,
                ),
                metrics,
            ),
        ),
        WebAuthMode::ProductionFailClosed => with_web_request_context(
            correlated,
            build_web_internal_api_framework_layer(
                MachineCredentialResolverDecorator::new(
                    ProductionFailClosedResolver,
                    machine_authenticator,
                ),
                metrics,
            ),
        ),
        WebAuthMode::IamDatabase(resolver) => with_web_request_context(
            correlated,
            build_web_internal_api_framework_layer(
                MachineCredentialResolverDecorator::new(resolver, machine_authenticator),
                metrics,
            ),
        ),
    }
}
