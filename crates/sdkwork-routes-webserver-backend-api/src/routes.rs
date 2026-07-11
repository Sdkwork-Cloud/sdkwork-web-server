use axum::{
    extract::{Path, Query, State},
    response::Response,
    routing::{get, post},
    Extension, Json, Router,
};
use sdkwork_webserver_contract::{
    CreateNginxConfigRequest, CreateServerRequest, ListNginxConfigsQuery, UpdateNginxConfigRequest,
    WebBackendApi, WebBackendRequestContext,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{agent_routes, auth::require_backend_context, paths};
use sdkwork_routes_webserver_common::{
    created_resource, ok_audit_log_page, ok_nginx_config_page, ok_resource, ok_server_page,
    WebApiError,
};

#[derive(Clone)]
struct BackendState {
    api: Arc<dyn WebBackendApi>,
}

pub fn build_router_with_backend_api<A>(api: A) -> Router
where
    A: WebBackendApi + 'static,
{
    build_router_with_shared_backend_api(Arc::new(api))
}

pub fn build_router_with_shared_backend_api(api: Arc<dyn WebBackendApi>) -> Router {
    Router::new()
        .route(
            paths::NGINX_CONFIGS,
            get(list_nginx_configs).post(create_nginx_config),
        )
        .route(
            paths::NGINX_CONFIG,
            get(retrieve_nginx_config).put(update_nginx_config),
        )
        .route(paths::NGINX_CONFIG_VALIDATE, post(validate_nginx_config))
        .route(paths::NGINX_CONFIG_DEPLOY, post(deploy_nginx_config))
        .route(paths::NGINX_RELOAD, post(reload_nginx))
        .route(paths::NGINX_STATUS, get(retrieve_nginx_status))
        .route(paths::SERVERS, get(list_servers).post(create_server))
        .route(paths::AUDIT_LOGS, get(list_audit_logs))
        // Agent routes (C8-C9): authenticated via X-SDKWork-Agent-Token through
        // the WebFrameworkLayer + AgentTokenResolverDecorator. Handlers retrieve
        // Arc<WebService> and WebBackendRequestContext from Extension layers.
        .route(paths::AGENT_HEARTBEAT, post(agent_routes::agent_heartbeat))
        .route(paths::AGENT_SYNC, get(agent_routes::agent_sync))
        .with_state(BackendState { api })
}

#[derive(Debug, Deserialize)]
struct PageQuery {
    #[serde(default = "default_page")]
    page: i32,
    #[serde(default = "default_page_size")]
    page_size: i32,
}

fn default_page() -> i32 {
    1
}

fn default_page_size() -> i32 {
    20
}

async fn list_nginx_configs(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
    Query(query): Query<ListNginxConfigsQuery>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    ok_nginx_config_page(state.api.list_nginx_configs(&context, &query).await)
}

async fn create_nginx_config(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
    Json(request): Json<CreateNginxConfigRequest>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    created_resource(state.api.create_nginx_config(&context, &request).await)
}

async fn retrieve_nginx_config(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
    Path(config_id): Path<String>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    ok_resource(state.api.retrieve_nginx_config(&context, &config_id).await)
}

async fn update_nginx_config(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
    Path(config_id): Path<String>,
    Json(request): Json<UpdateNginxConfigRequest>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    ok_resource(
        state
            .api
            .update_nginx_config(&context, &config_id, &request)
            .await,
    )
}

async fn validate_nginx_config(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
    Path(config_id): Path<String>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    ok_resource(state.api.validate_nginx_config(&context, &config_id).await)
}

async fn deploy_nginx_config(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
    Path(config_id): Path<String>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    ok_resource(state.api.web_nginx_config(&context, &config_id).await)
}

async fn reload_nginx(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    ok_resource(state.api.reload_nginx(&context).await)
}

async fn retrieve_nginx_status(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    ok_resource(state.api.retrieve_nginx_status(&context).await)
}

async fn list_servers(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
    Query(query): Query<PageQuery>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    ok_server_page(
        state
            .api
            .list_servers(&context, query.page, query.page_size)
            .await,
        query.page,
        query.page_size,
    )
}

async fn create_server(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
    Json(request): Json<CreateServerRequest>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    created_resource(state.api.create_server(&context, &request).await)
}

async fn list_audit_logs(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
    Query(query): Query<PageQuery>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    ok_audit_log_page(
        state
            .api
            .list_audit_logs(&context, query.page, query.page_size)
            .await,
    )
}
