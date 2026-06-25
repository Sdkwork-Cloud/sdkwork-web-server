use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use sdkwork_webserver_contract::{
    CreateCertificateRequest, CreateDeploymentRequest, CreateDomainRequest,
    CreateEnvVariableRequest, CreateHealthCheckRequest, CreateSiteRequest, ListSitesQuery,
    UpdateSiteRequest, WebAppApi, WebAppRequestContext, WebServiceResult,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{auth::require_app_context, paths};
use sdkwork_router_webserver_common::WebApiError;

#[derive(Clone)]
struct AppState {
    api: Arc<dyn WebAppApi>,
}

pub fn build_router_with_app_api<A>(api: A) -> Router
where
    A: WebAppApi + 'static,
{
    build_router_with_shared_app_api(Arc::new(api))
}

pub fn build_router_with_shared_app_api(api: Arc<dyn WebAppApi>) -> Router {
    Router::new()
        .route(paths::SITES, get(list_sites).post(create_site))
        .route(
            paths::SITE,
            get(retrieve_site).patch(update_site).delete(delete_site),
        )
        .route(paths::SITE_ACTIVATE, post(activate_site))
        .route(paths::SITE_PAUSE, post(pause_site))
        .route(paths::SITE_DOMAINS, get(list_domains).post(create_domain))
        .route(
            paths::SITE_DOMAIN,
            get(retrieve_domain).delete(delete_domain),
        )
        .route(paths::SITE_DOMAIN_VERIFY, post(verify_domain))
        .route(
            paths::SITE_DEPLOYMENTS,
            get(list_deployments).post(create_deployment),
        )
        .route(paths::SITE_DEPLOYMENT, get(retrieve_deployment))
        .route(paths::SITE_DEPLOYMENT_ROLLBACK, post(rollback_deployment))
        .route(
            paths::SITE_ENV_VARIABLES,
            get(list_env_variables).post(create_env_variable),
        )
        .route(
            paths::CERTIFICATES,
            get(list_certificates).post(create_certificate),
        )
        .route(
            paths::SITE_HEALTH_CHECKS,
            get(list_health_checks).post(create_health_check),
        )
        .with_state(AppState { api })
}

#[derive(Debug, Deserialize)]
struct PageQuery {
    #[serde(default = "default_page")]
    page: i32,
    #[serde(default = "default_page_size", rename = "pageSize")]
    page_size: i32,
}

#[derive(Debug, Deserialize)]
struct DeploymentListQuery {
    #[serde(default = "default_page")]
    page: i32,
    #[serde(default = "default_page_size", rename = "pageSize")]
    page_size: i32,
    status: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct EnvVariableListQuery {
    environment: Option<String>,
}

fn default_page() -> i32 {
    1
}

fn default_page_size() -> i32 {
    20
}

async fn list_sites(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Query(query): Query<ListSitesQuery>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    ok_json(state.api.list_sites(&context, &query).await)
}

async fn create_site(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Json(request): Json<CreateSiteRequest>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    created_json(state.api.create_site(&context, &request).await)
}

async fn retrieve_site(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path(site_id): Path<String>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    ok_json(state.api.retrieve_site(&context, &site_id).await)
}

async fn update_site(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path(site_id): Path<String>,
    Json(request): Json<UpdateSiteRequest>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    ok_json(state.api.update_site(&context, &site_id, &request).await)
}

async fn delete_site(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path(site_id): Path<String>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    no_content(state.api.delete_site(&context, &site_id).await)
}

async fn activate_site(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path(site_id): Path<String>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    ok_json(state.api.activate_site(&context, &site_id).await)
}

async fn pause_site(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path(site_id): Path<String>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    ok_json(state.api.pause_site(&context, &site_id).await)
}

async fn list_domains(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path(site_id): Path<String>,
    Query(query): Query<PageQuery>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .list_domains(&context, &site_id, query.page, query.page_size)
            .await,
    )
}

async fn create_domain(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path(site_id): Path<String>,
    Json(request): Json<CreateDomainRequest>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    created_json(state.api.create_domain(&context, &site_id, &request).await)
}

async fn retrieve_domain(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path((site_id, domain_id)): Path<(String, String)>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .retrieve_domain(&context, &site_id, &domain_id)
            .await,
    )
}

async fn delete_domain(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path((site_id, domain_id)): Path<(String, String)>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    no_content(
        state
            .api
            .delete_domain(&context, &site_id, &domain_id)
            .await,
    )
}

async fn verify_domain(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path((site_id, domain_id)): Path<(String, String)>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .verify_domain(&context, &site_id, &domain_id)
            .await,
    )
}

async fn list_deployments(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path(site_id): Path<String>,
    Query(query): Query<DeploymentListQuery>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .list_deployments(
                &context,
                &site_id,
                query.page,
                query.page_size,
                query.status,
            )
            .await,
    )
}

async fn create_deployment(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path(site_id): Path<String>,
    Json(request): Json<CreateDeploymentRequest>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    created_json(
        state
            .api
            .create_deployment(&context, &site_id, &request)
            .await,
    )
}

async fn retrieve_deployment(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path((site_id, deployment_id)): Path<(String, String)>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .retrieve_deployment(&context, &site_id, &deployment_id)
            .await,
    )
}

async fn rollback_deployment(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path((site_id, deployment_id)): Path<(String, String)>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .rollback_deployment(&context, &site_id, &deployment_id)
            .await,
    )
}

async fn list_env_variables(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path(site_id): Path<String>,
    Query(query): Query<EnvVariableListQuery>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .list_env_variables(&context, &site_id, query.environment.as_deref())
            .await,
    )
}

async fn create_env_variable(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path(site_id): Path<String>,
    Json(request): Json<CreateEnvVariableRequest>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    created_json(
        state
            .api
            .create_env_variable(&context, &site_id, &request)
            .await,
    )
}

async fn list_certificates(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Query(query): Query<PageQuery>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .list_certificates(&context, query.page, query.page_size)
            .await,
    )
}

async fn create_certificate(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Json(request): Json<CreateCertificateRequest>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    created_json(state.api.create_certificate(&context, &request).await)
}

async fn list_health_checks(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path(site_id): Path<String>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    ok_json(state.api.list_health_checks(&context, &site_id).await)
}

async fn create_health_check(
    State(state): State<AppState>,
    context: Option<Extension<WebAppRequestContext>>,
    Path(site_id): Path<String>,
    Json(request): Json<CreateHealthCheckRequest>,
) -> Result<Response, WebApiError> {
    let context = require_app_context(context)?;
    created_json(
        state
            .api
            .create_health_check(&context, &site_id, &request)
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

fn no_content(result: WebServiceResult<()>) -> Result<Response, WebApiError> {
    match result {
        Ok(()) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(error) => Err(error.into()),
    }
}
