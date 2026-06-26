use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sdkwork_webserver_contract::WebServiceError;

use crate::correlation::WebProblemCorrelation;

pub type WebApiResult<T> = Result<T, WebApiError>;

#[derive(Debug, Clone)]
pub struct WebApiError {
    status: StatusCode,
    code: String,
    detail: String,
}

impl WebApiError {
    pub fn new(status: StatusCode, code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            status,
            code: code.into(),
            detail: detail.into(),
        }
    }
}

impl From<WebServiceError> for WebApiError {
    fn from(error: WebServiceError) -> Self {
        use sdkwork_webserver_contract::WebServiceErrorKind;
        let (status, code) = match error.kind() {
            WebServiceErrorKind::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            WebServiceErrorKind::Conflict => (StatusCode::CONFLICT, "conflict"),
            WebServiceErrorKind::Validation => {
                (StatusCode::UNPROCESSABLE_ENTITY, "validation_error")
            }
            WebServiceErrorKind::Forbidden => (StatusCode::FORBIDDEN, "forbidden"),
            WebServiceErrorKind::DatabaseUnavailable => {
                (StatusCode::SERVICE_UNAVAILABLE, "database_unavailable")
            }
            WebServiceErrorKind::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };
        Self::new(status, code, error.to_string())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WebApiProblem {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

impl WebApiProblem {
    pub fn from_error(error: &WebApiError) -> Self {
        let correlation = WebProblemCorrelation::current().unwrap_or_default();
        Self {
            problem_type: format!("https://sdkwork.com/problems/{}", error.code),
            title: error.code.clone(),
            status: error.status.as_u16(),
            detail: error.detail.clone(),
            instance: None,
            request_id: Some(correlation.request_id),
            trace_id: correlation.trace_id,
        }
    }
}

impl IntoResponse for WebApiError {
    fn into_response(self) -> Response {
        let problem = WebApiProblem::from_error(&self);
        let mut response = (self.status, Json(problem)).into_response();
        if let Some(correlation) = WebProblemCorrelation::current() {
            if let Ok(value) = HeaderValue::from_str(&correlation.request_id) {
                response
                    .headers_mut()
                    .insert(header::HeaderName::from_static("x-request-id"), value);
            }
        }
        response
    }
}
