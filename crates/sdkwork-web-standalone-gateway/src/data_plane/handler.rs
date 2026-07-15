use std::net::SocketAddr;

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{
        header::{CONTENT_LENGTH, CONTENT_TYPE, HOST, LOCATION},
        HeaderValue, Request, Response, StatusCode, Version,
    },
};
use sdkwork_webserver_core::{normalize_authority_host, ResourceConfig};

use super::{
    proxy::{proxy_request, text_response},
    static_files::serve_static,
    ListenerState,
};

pub async fn route_request(
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    State(state): State<ListenerState>,
    request: Request<Body>,
) -> Response<Body> {
    let authority = match request_authority(&request) {
        Ok(authority) => authority,
        Err((status, message)) => return text_response(status, message),
    };
    let method = request.method().as_str().to_owned();
    let path = request.uri().path().to_owned();
    let Some(selected) =
        state
            .runtime
            .app
            .select_route(&state.listener_id, &authority, &path, &method)
    else {
        return text_response(StatusCode::NOT_FOUND, "route was not found\n");
    };

    let virtual_host_id = selected.virtual_host.id.clone();
    let route_id = selected.route.id.clone();
    let response = match selected.resource {
        ResourceConfig::Respond {
            status,
            content_type,
            body,
            ..
        } => fixed_response(*status, content_type, body, method == "HEAD"),
        ResourceConfig::Redirect {
            status, location, ..
        } => redirect_response(*status, location),
        ResourceConfig::Static {
            id, spa_fallback, ..
        } => {
            let Some(root) = state.runtime.app.static_root(id) else {
                return text_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "static resource is unavailable\n",
                );
            };
            serve_static(root, selected.route, spa_fallback.as_deref(), request).await
        }
        ResourceConfig::Proxy {
            upstream_ref,
            strip_prefix,
            ..
        } => {
            let scheme = if state.is_tls { "https" } else { "http" };
            proxy_request(
                super::proxy::ProxyRequestContext {
                    runtime: &state.runtime,
                    upstream_ref,
                    strip_prefix: *strip_prefix,
                    route: selected.route,
                    peer,
                    external_scheme: scheme,
                    external_authority: &authority,
                },
                request,
            )
            .await
        }
    };

    if state.runtime.app.config().observability.access_log {
        tracing::info!(
            listener_id = %state.listener_id,
            virtual_host_id = %virtual_host_id,
            route_id = %route_id,
            method = %method,
            status = response.status().as_u16(),
            "request served"
        );
    }
    response
}

fn request_authority(request: &Request<Body>) -> Result<String, (StatusCode, &'static str)> {
    let mut host_values = request.headers().get_all(HOST).iter();
    let header_authority = match host_values.next() {
        Some(value) => Some(
            value
                .to_str()
                .map_err(|_| (StatusCode::BAD_REQUEST, "invalid Host header\n"))?,
        ),
        None => None,
    };
    if host_values.next().is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            "multiple Host headers are forbidden\n",
        ));
    }

    let uri_authority = request.uri().authority().map(|value| value.as_str());
    if let (Some(uri), Some(header)) = (uri_authority, header_authority) {
        if normalize_authority_host(uri) != normalize_authority_host(header) {
            return Err((
                StatusCode::BAD_REQUEST,
                "request authority conflicts with Host\n",
            ));
        }
    }
    let authority = uri_authority.or(header_authority).unwrap_or_default();
    if matches!(request.version(), Version::HTTP_11 | Version::HTTP_2)
        && normalize_authority_host(authority).is_none()
    {
        return Err((StatusCode::BAD_REQUEST, "request authority is required\n"));
    }
    Ok(authority.to_owned())
}

fn fixed_response(status: u16, content_type: &str, body: &str, head: bool) -> Response<Body> {
    let status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let suppress_body =
        head || status == StatusCode::NO_CONTENT || status == StatusCode::NOT_MODIFIED;
    let mut response = Response::new(if suppress_body {
        Body::empty()
    } else {
        Body::from(body.to_owned())
    });
    *response.status_mut() = status;
    if let Ok(value) = HeaderValue::from_str(content_type) {
        response.headers_mut().insert(CONTENT_TYPE, value);
    }
    let declared_length = if matches!(status, StatusCode::NO_CONTENT | StatusCode::RESET_CONTENT) {
        0
    } else {
        body.len()
    };
    if let Ok(value) = HeaderValue::from_str(&declared_length.to_string()) {
        response.headers_mut().insert(CONTENT_LENGTH, value);
    }
    response
}

fn redirect_response(status: u16, location: &str) -> Response<Body> {
    let status = StatusCode::from_u16(status).unwrap_or(StatusCode::TEMPORARY_REDIRECT);
    let mut response = Response::new(Body::empty());
    *response.status_mut() = status;
    if let Ok(value) = HeaderValue::from_str(location) {
        response.headers_mut().insert(LOCATION, value);
    }
    response
}
