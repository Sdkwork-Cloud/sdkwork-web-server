use axum::Extension;

use sdkwork_routes_webserver_common::WebApiError;
use sdkwork_webserver_contract::WebBackendRequestContext;

pub fn require_backend_context(
    context: Option<Extension<WebBackendRequestContext>>,
) -> Result<WebBackendRequestContext, WebApiError> {
    context.map(|Extension(context)| context).ok_or_else(|| {
        WebApiError::authentication_required("authenticated backend request context is required")
    })
}
