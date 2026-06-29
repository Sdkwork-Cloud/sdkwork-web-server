use axum::Extension;

use sdkwork_routes_webserver_common::WebApiError;
use sdkwork_webserver_contract::WebAppRequestContext;

pub fn require_app_context(
    context: Option<Extension<WebAppRequestContext>>,
) -> Result<WebAppRequestContext, WebApiError> {
    context.map(|Extension(context)| context).ok_or_else(|| {
        WebApiError::authentication_required("authenticated app request context is required")
    })
}
