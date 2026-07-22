use axum::Extension;

use sdkwork_routes_webserver_common::WebApiError;
use sdkwork_webserver_contract::WebInternalRequestContext;

pub fn require_internal_context(
    context: Option<Extension<WebInternalRequestContext>>,
) -> Result<WebInternalRequestContext, WebApiError> {
    context.map(|Extension(context)| context).ok_or_else(|| {
        WebApiError::authentication_required("authenticated internal request context is required")
    })
}
