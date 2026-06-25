//! Agent API routes authenticated via X-SDKWork-Agent-Token (outside IAM dual-token).

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use sdkwork_intelligence_webserver_service::WebService;
use sdkwork_router_webserver_common::WebApiError;
use sdkwork_webserver_contract::{AgentHeartbeatRequest, WebServiceError, WebServiceResult};
use std::sync::Arc;

pub const AGENT_HEARTBEAT: &str = "/backend/v3/api/agent/heartbeat";
pub const AGENT_SYNC: &str = "/backend/v3/api/agent/sync";
const AGENT_TOKEN_HEADER: &str = "x-sdkwork-agent-token";

#[derive(Debug, serde::Deserialize)]
struct AgentSyncQuery {
    #[serde(rename = "ifSyncVersion")]
    if_sync_version: Option<String>,
}

#[derive(Clone)]
struct AgentState {
    service: Arc<WebService>,
}

pub fn build_agent_router(service: Arc<WebService>) -> Router {
    Router::new()
        .route(AGENT_HEARTBEAT, post(agent_heartbeat))
        .route(AGENT_SYNC, get(agent_sync))
        .with_state(AgentState { service })
}

async fn agent_heartbeat(
    State(state): State<AgentState>,
    headers: HeaderMap,
    Json(request): Json<AgentHeartbeatRequest>,
) -> Result<Response, WebApiError> {
    let token = extract_agent_token(&headers)?;
    ok_json(state.service.agent_heartbeat(&token, &request).await)
}

async fn agent_sync(
    State(state): State<AgentState>,
    headers: HeaderMap,
    Query(query): Query<AgentSyncQuery>,
) -> Result<Response, WebApiError> {
    let token = extract_agent_token(&headers)?;
    ok_json(
        state
            .service
            .agent_sync(
                &token,
                query
                    .if_sync_version
                    .as_deref()
                    .filter(|value| !value.is_empty()),
            )
            .await,
    )
}

fn extract_agent_token(headers: &HeaderMap) -> Result<String, WebApiError> {
    headers
        .get(AGENT_TOKEN_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            WebApiError::new(
                StatusCode::UNAUTHORIZED,
                "missing_agent_token",
                "X-SDKWork-Agent-Token header is required",
            )
        })
}

fn ok_json<T>(result: WebServiceResult<T>) -> Result<Response, WebApiError>
where
    T: serde::Serialize,
{
    match result {
        Ok(value) => Ok((StatusCode::OK, Json(value)).into_response()),
        Err(WebServiceError::Forbidden) => Err(WebApiError::new(
            StatusCode::UNAUTHORIZED,
            "invalid_agent_token",
            "agent token is invalid",
        )),
        Err(error) => Err(error.into()),
    }
}
