use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use sdkwork_webserver_contract::{
    CreateNginxConfigRequest, CreateServerRequest, ListNginxConfigsQuery, UpdateNginxConfigRequest,
    WebBackendApi, WebBackendRequestContext, WebServiceResult,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{auth::require_backend_context, paths};
use sdkwork_router_webserver_common::WebApiError;

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
        .with_state(BackendState { api })
}

#[derive(Debug, Deserialize)]
struct PageQuery {
    #[serde(default = "default_page")]
    page: i32,
    #[serde(default = "default_page_size", rename = "pageSize")]
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
    ok_json(state.api.list_nginx_configs(&context, &query).await)
}

async fn create_nginx_config(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
    Json(request): Json<CreateNginxConfigRequest>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    created_json(state.api.create_nginx_config(&context, &request).await)
}

async fn retrieve_nginx_config(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
    Path(config_id): Path<String>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    ok_json(state.api.retrieve_nginx_config(&context, &config_id).await)
}

async fn update_nginx_config(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
    Path(config_id): Path<String>,
    Json(request): Json<UpdateNginxConfigRequest>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    ok_json(
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
    ok_json(state.api.validate_nginx_config(&context, &config_id).await)
}

async fn deploy_nginx_config(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
    Path(config_id): Path<String>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    ok_json(state.api.web_nginx_config(&context, &config_id).await)
}

async fn reload_nginx(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    ok_json(state.api.reload_nginx(&context).await)
}

async fn retrieve_nginx_status(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    ok_json(state.api.retrieve_nginx_status(&context).await)
}

async fn list_servers(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
    Query(query): Query<PageQuery>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    ok_json(
        state
            .api
            .list_servers(&context, query.page, query.page_size)
            .await,
    )
}

async fn create_server(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
    Json(request): Json<CreateServerRequest>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    created_json(state.api.create_server(&context, &request).await)
}

async fn list_audit_logs(
    State(state): State<BackendState>,
    context: Option<Extension<WebBackendRequestContext>>,
    Query(query): Query<PageQuery>,
) -> Result<Response, WebApiError> {
    let context = require_backend_context(context)?;
    ok_json(
        state
            .api
            .list_audit_logs(&context, query.page, query.page_size)
            .await,
    )
}

fn ok_json<T>(result: WebServiceResult<T>) -> Result<Response, WebApiError>
where
    T: serde::Serialize,
{
    match result {
        Ok(value) => Ok((StatusCode::OK, Json(value)).into_response()),
        Err(error) => Err(error.into()),
    }
}

fn created_json<T>(result: WebServiceResult<T>) -> Result<Response, WebApiError>
where
    T: serde::Serialize,
{
    match result {
        Ok(value) => Ok((StatusCode::CREATED, Json(value)).into_response()),
        Err(error) => Err(error.into()),
    }
}
