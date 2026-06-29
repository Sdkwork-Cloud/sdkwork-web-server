//! Agent API routes integrated into the backend-api WebFrameworkLayer pipeline (C8-C9).
//!
//! Agent routes (`/backend/v3/api/agent/heartbeat`, `/backend/v3/api/agent/sync`) are
//! declared with `RouteAuth::AgentToken` in the route manifest. The framework
//! authenticates `X-SDKWork-Agent-Token` via `AgentTokenResolverDecorator::resolve_api_key`
//! and injects `WebBackendRequestContext` with `tenant_id` and `subject_id` (server UUID).
//! Handlers retrieve `Arc<WebService>` from `Extension` (applied in `web_bootstrap.rs`).

use axum::{
    extract::{Extension, Query},
    response::Response,
    Json,
};
use sdkwork_intelligence_webserver_service::WebService;
use sdkwork_routes_webserver_common::{ok_resource, WebApiError};
use sdkwork_utils_rust::SdkWorkResultCode;
use sdkwork_webserver_contract::{
    AgentHeartbeatRequest, WebBackendRequestContext, WebServiceError,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub(crate) struct AgentSyncQuery {
    #[serde(rename = "ifSyncVersion")]
    if_sync_version: Option<String>,
}

pub(crate) async fn agent_heartbeat(
    Extension(service): Extension<Arc<WebService>>,
    Extension(context): Extension<WebBackendRequestContext>,
    Json(request): Json<AgentHeartbeatRequest>,
) -> Result<Response, WebApiError> {
    let (server_id, tenant_id) = require_agent_context(&context)?;
    map_agent_result(
        service
            .agent_heartbeat(server_id, tenant_id, &request)
            .await,
    )
}

pub(crate) async fn agent_sync(
    Extension(service): Extension<Arc<WebService>>,
    Extension(context): Extension<WebBackendRequestContext>,
    Query(query): Query<AgentSyncQuery>,
) -> Result<Response, WebApiError> {
    let (server_id, tenant_id) = require_agent_context(&context)?;
    map_agent_result(
        service
            .agent_sync(
                server_id,
                tenant_id,
                query
                    .if_sync_version
                    .as_deref()
                    .filter(|value| !value.is_empty()),
            )
            .await,
    )
}

/// Extracts `(server_uuid, tenant_id)` from the framework-injected backend context.
///
/// `subject_id` holds the principal's `user_id` (server UUID for agent-token routes).
/// `tenant_id` is guaranteed by the fail-closed injector.
fn require_agent_context(context: &WebBackendRequestContext) -> Result<(&str, i64), WebApiError> {
    let server_id = context.subject_id.as_deref().ok_or_else(|| {
        WebApiError::authentication_required("agent route requires an authenticated server subject")
    })?;
    let tenant_id = context.tenant_id.ok_or_else(|| {
        WebApiError::authentication_required("agent route requires tenant isolation context")
    })?;
    Ok((server_id, tenant_id))
}

fn map_agent_result<T>(result: Result<T, WebServiceError>) -> Result<Response, WebApiError>
where
    T: serde::Serialize,
{
    match result {
        Ok(value) => ok_resource(Ok(value)),
        Err(WebServiceError::Forbidden) => Err(WebApiError::new(
            SdkWorkResultCode::InvalidToken,
            "agent token is invalid or has been revoked",
        )),
        Err(error) => Err(error.into()),
    }
}
