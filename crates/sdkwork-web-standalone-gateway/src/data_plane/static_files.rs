use std::{convert::Infallible, io, path::Path};

use axum::{
    body::Body,
    http::{Request, Response, StatusCode, Uri},
};
use percent_encoding::percent_decode_str;
use sdkwork_webserver_core::{RouteConfig, RoutePathType};
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

use super::proxy::text_response;

pub async fn serve_static(
    root: &Path,
    route: &RouteConfig,
    spa_fallback: Option<&str>,
    request: Request<Body>,
) -> Response<Body> {
    if !matches!(request.method().as_str(), "GET" | "HEAD") {
        return text_response(StatusCode::METHOD_NOT_ALLOWED, "method is not allowed\n");
    }

    let relative = relative_request_path(route, request.uri().path()).to_owned();
    let decoded = match percent_decode_str(&relative).decode_utf8() {
        Ok(decoded) => decoded,
        Err(_) => return text_response(StatusCode::BAD_REQUEST, "invalid request path\n"),
    };
    if let Err(status) = verify_static_path(root, &decoded).await {
        return text_response(status, "static path is not available\n");
    }

    let (mut parts, _) = request.into_parts();
    let service_path = format!("/{}", relative.trim_start_matches('/'));
    parts.uri = match service_path.parse::<Uri>() {
        Ok(uri) => uri,
        Err(_) => return text_response(StatusCode::BAD_REQUEST, "invalid request path\n"),
    };
    let request = Request::from_parts(parts, Body::empty());

    if let Some(fallback) = spa_fallback {
        map_file_response(
            ServeDir::new(root)
                .append_index_html_on_directories(true)
                .not_found_service(ServeFile::new(root.join(fallback)))
                .oneshot(request)
                .await,
        )
    } else {
        map_file_response(
            ServeDir::new(root)
                .append_index_html_on_directories(true)
                .oneshot(request)
                .await,
        )
    }
}

fn relative_request_path<'a>(route: &RouteConfig, request_path: &'a str) -> &'a str {
    match route.route_match.path_type {
        RoutePathType::Exact | RoutePathType::Prefix => request_path
            .strip_prefix(&route.route_match.path)
            .unwrap_or(request_path),
    }
}

async fn verify_static_path(root: &Path, relative: &str) -> Result<(), StatusCode> {
    if relative.contains('\\') || relative.contains('\0') {
        return Err(StatusCode::BAD_REQUEST);
    }
    let relative = relative.trim_start_matches('/');
    let path = Path::new(relative);
    if path.components().any(|component| {
        !matches!(
            component,
            std::path::Component::Normal(_) | std::path::Component::CurDir
        )
    }) {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut current = root.to_path_buf();
    for component in path.components() {
        let std::path::Component::Normal(component) = component else {
            continue;
        };
        current.push(component);
        match tokio::fs::symlink_metadata(&current).await {
            Ok(metadata) if metadata.file_type().is_symlink() => return Err(StatusCode::FORBIDDEN),
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => break,
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
    Ok(())
}

fn map_file_response<B>(result: Result<Response<B>, Infallible>) -> Response<Body>
where
    B: http_body::Body<Data = bytes::Bytes> + Send + 'static,
    B::Error: Into<axum::BoxError>,
{
    match result {
        Ok(response) => response.map(Body::new),
        Err(never) => match never {},
    }
}
