use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use sdkwork_utils_rust::{
    PageInfo, PageMode, SdkWorkApiResponse, SdkWorkPageData, SdkWorkResourceData,
    SDKWORK_TRACE_ID_HEADER,
};
use sdkwork_web_core::new_request_id;
use sdkwork_webserver_contract::{
    AuditLogPage, CertificatePage, DeploymentPage, DomainPage, EnvVariablePage, HealthCheckPage,
    NginxConfigPage, ServerPage, SitePage, WebServiceResult,
};
use serde::Serialize;

use crate::{correlation::WebProblemCorrelation, WebApiError};

fn resolved_trace_id() -> String {
    WebProblemCorrelation::current()
        .and_then(|correlation| correlation.trace_id.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(new_request_id)
}

fn attach_trace_header(response: &mut Response, trace_id: &str) {
    if let (Ok(name), Ok(value)) = (
        HeaderName::from_bytes(SDKWORK_TRACE_ID_HEADER.as_bytes()),
        HeaderValue::from_str(trace_id),
    ) {
        response.headers_mut().insert(name, value);
    }
}

fn envelope<T: Serialize>(status: StatusCode, data: T) -> Response {
    let trace_id = resolved_trace_id();
    let body = SdkWorkApiResponse::success(data, trace_id.clone());
    let mut response = (status, Json(body)).into_response();
    attach_trace_header(&mut response, &trace_id);
    response
}

fn offset_page_info(page: i32, page_size: i32, total: i64) -> PageInfo {
    PageInfo {
        mode: PageMode::Offset,
        page: Some(page),
        page_size: Some(page_size),
        total_items: Some(total.to_string()),
        total_pages: None,
        next_cursor: None,
        has_more: None,
    }
}

fn build_page_data<T: Serialize>(
    items: Vec<T>,
    page: i32,
    page_size: i32,
    total: i64,
) -> SdkWorkPageData<T> {
    SdkWorkPageData {
        items,
        page_info: offset_page_info(page, page_size, total),
    }
}

pub fn ok_resource<T: Serialize>(result: WebServiceResult<T>) -> Result<Response, WebApiError> {
    match result {
        Ok(item) => Ok(envelope(StatusCode::OK, SdkWorkResourceData { item })),
        Err(error) => Err(error.into()),
    }
}

pub fn created_resource<T: Serialize>(
    result: WebServiceResult<T>,
) -> Result<Response, WebApiError> {
    match result {
        Ok(item) => Ok(envelope(StatusCode::CREATED, SdkWorkResourceData { item })),
        Err(error) => Err(error.into()),
    }
}

pub fn ok_site_page(result: WebServiceResult<SitePage>) -> Result<Response, WebApiError> {
    match result {
        Ok(page) => Ok(envelope(
            StatusCode::OK,
            build_page_data(page.items, page.page, page.page_size, page.total),
        )),
        Err(error) => Err(error.into()),
    }
}

pub fn ok_deployment_page(
    result: WebServiceResult<DeploymentPage>,
) -> Result<Response, WebApiError> {
    match result {
        Ok(page) => Ok(envelope(
            StatusCode::OK,
            build_page_data(page.items, page.page, page.page_size, page.total),
        )),
        Err(error) => Err(error.into()),
    }
}

pub fn ok_nginx_config_page(
    result: WebServiceResult<NginxConfigPage>,
) -> Result<Response, WebApiError> {
    match result {
        Ok(page) => Ok(envelope(
            StatusCode::OK,
            build_page_data(page.items, page.page, page.page_size, page.total),
        )),
        Err(error) => Err(error.into()),
    }
}

pub fn ok_audit_log_page(result: WebServiceResult<AuditLogPage>) -> Result<Response, WebApiError> {
    match result {
        Ok(page) => Ok(envelope(
            StatusCode::OK,
            build_page_data(page.items, page.page, page.page_size, page.total),
        )),
        Err(error) => Err(error.into()),
    }
}

pub fn ok_domain_page(
    result: WebServiceResult<DomainPage>,
    page: i32,
    page_size: i32,
) -> Result<Response, WebApiError> {
    match result {
        Ok(page_data) => Ok(envelope(
            StatusCode::OK,
            build_page_data(page_data.items, page, page_size, page_data.total),
        )),
        Err(error) => Err(error.into()),
    }
}

pub fn ok_env_variable_page(
    result: WebServiceResult<EnvVariablePage>,
) -> Result<Response, WebApiError> {
    match result {
        Ok(page) => {
            let page_size = page.items.len().max(1) as i32;
            Ok(envelope(
                StatusCode::OK,
                build_page_data(page.items, 1, page_size, page.total),
            ))
        }
        Err(error) => Err(error.into()),
    }
}

pub fn ok_certificate_page(
    result: WebServiceResult<CertificatePage>,
    page: i32,
    page_size: i32,
) -> Result<Response, WebApiError> {
    match result {
        Ok(page_data) => Ok(envelope(
            StatusCode::OK,
            build_page_data(page_data.items, page, page_size, page_data.total),
        )),
        Err(error) => Err(error.into()),
    }
}

pub fn ok_health_check_page(
    result: WebServiceResult<HealthCheckPage>,
) -> Result<Response, WebApiError> {
    match result {
        Ok(page) => {
            let page_size = page.items.len().max(1) as i32;
            Ok(envelope(
                StatusCode::OK,
                build_page_data(page.items, 1, page_size, page.total),
            ))
        }
        Err(error) => Err(error.into()),
    }
}

pub fn ok_server_page(
    result: WebServiceResult<ServerPage>,
    page: i32,
    page_size: i32,
) -> Result<Response, WebApiError> {
    match result {
        Ok(page_data) => Ok(envelope(
            StatusCode::OK,
            build_page_data(page_data.items, page, page_size, page_data.total),
        )),
        Err(error) => Err(error.into()),
    }
}

pub fn no_content(result: WebServiceResult<()>) -> Result<Response, WebApiError> {
    match result {
        Ok(()) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;
    use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkResourceData, SDKWORK_SUCCESS_CODE};
    use sdkwork_webserver_contract::AgentSyncResponse;

    use super::ok_resource;

    #[tokio::test]
    async fn agent_sync_resource_uses_the_canonical_sdkwork_envelope() {
        let manifest = AgentSyncResponse {
            server_id: "server-1".to_string(),
            sync_version: "sv1:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            unchanged: true,
            nginx_configs: Vec::new(),
            certificates: Vec::new(),
        };
        let response = ok_resource(Ok(manifest)).expect("resource response");
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("bounded response body");
        let decoded: SdkWorkApiResponse<SdkWorkResourceData<AgentSyncResponse>> =
            serde_json::from_slice(&body).expect("canonical resource envelope");

        assert_eq!(decoded.code, SDKWORK_SUCCESS_CODE);
        assert!(!decoded.trace_id.is_empty());
        assert_eq!(decoded.data.item.server_id, "server-1");
        assert!(decoded.data.item.unchanged);
    }
}
