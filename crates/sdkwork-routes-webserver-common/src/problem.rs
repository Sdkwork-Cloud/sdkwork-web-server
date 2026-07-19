use axum::{
    http::{header, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sdkwork_utils_rust::{SdkWorkProblemDetail, SdkWorkResultCode, SDKWORK_TRACE_ID_HEADER};
use sdkwork_web_core::new_request_id;
use sdkwork_webserver_contract::WebServiceError;

use crate::correlation::WebProblemCorrelation;

pub type WebApiResult<T> = Result<T, WebApiError>;

#[derive(Debug, Clone)]
pub struct WebApiError {
    code: SdkWorkResultCode,
    detail: String,
}

impl WebApiError {
    pub fn new(code: SdkWorkResultCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }

    pub fn authentication_required(detail: impl Into<String>) -> Self {
        Self::new(SdkWorkResultCode::AuthenticationRequired, detail)
    }
}

impl From<WebServiceError> for WebApiError {
    fn from(error: WebServiceError) -> Self {
        use sdkwork_webserver_contract::WebServiceErrorKind;
        let code = match error.kind() {
            WebServiceErrorKind::NotFound => SdkWorkResultCode::NotFound,
            WebServiceErrorKind::Conflict => SdkWorkResultCode::Conflict,
            WebServiceErrorKind::Validation => SdkWorkResultCode::ValidationError,
            WebServiceErrorKind::Forbidden => SdkWorkResultCode::PermissionRequired,
            WebServiceErrorKind::DatabaseUnavailable => SdkWorkResultCode::ServiceUnavailable,
            WebServiceErrorKind::Internal => SdkWorkResultCode::InternalError,
        };
        Self::new(code, error.to_string())
    }
}

fn resolved_trace_id() -> String {
    WebProblemCorrelation::current()
        .and_then(|correlation| correlation.trace_id.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(new_request_id)
}

impl IntoResponse for WebApiError {
    fn into_response(self) -> Response {
        let trace_id = resolved_trace_id();
        let problem = SdkWorkProblemDetail::platform(self.code, self.detail, trace_id.clone());
        let status =
            StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut response = (
            status,
            [(header::CONTENT_TYPE, "application/problem+json")],
            Json(problem),
        )
            .into_response();
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(SDKWORK_TRACE_ID_HEADER.as_bytes()),
            HeaderValue::from_str(&trace_id),
        ) {
            response.headers_mut().insert(name, value);
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;
    use sdkwork_utils_rust::{SdkWorkResultCode, SDKWORK_TRACE_ID_HEADER};

    use super::WebApiError;

    #[test]
    fn problem_response_adds_trace_header_without_panicking() {
        let response =
            WebApiError::new(SdkWorkResultCode::ValidationError, "invalid request").into_response();

        assert!(response.headers().get(SDKWORK_TRACE_ID_HEADER).is_some());
    }
}
