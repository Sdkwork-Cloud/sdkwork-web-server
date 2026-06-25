use axum::{http::StatusCode, Extension};

use sdkwork_router_webserver_common::WebApiError;
use sdkwork_webserver_contract::WebBackendRequestContext;

pub fn require_backend_context(
    context: Option<Extension<WebBackendRequestContext>>,
) -> Result<WebBackendRequestContext, WebApiError> {
    context.map(|Extension(context)| context).ok_or_else(|| {
        WebApiError::new(
            StatusCode::UNAUTHORIZED,
            "missing_backend_request_context",
            "authenticated backend request context is required",
        )
    })
}
