use std::{net::SocketAddr, time::Duration};

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{
        header::{
            CONNECTION, CONTENT_LENGTH, CONTENT_TYPE, EXPECT, HOST, LOCATION, RETRY_AFTER, TE,
            TRANSFER_ENCODING,
        },
        HeaderMap, HeaderValue, Request, Response, StatusCode, Version,
    },
};
use futures_util::StreamExt;
use sdkwork_webserver_core::{normalize_authority_host, ResourceConfig, RoutePathType};

use super::{
    metrics::RequestRejection,
    proxy::{proxy_request, request_body_timeout_response, text_response},
    proxy_body::RequestBodyFailure,
    proxy_protocol::DownstreamConnectionInfo,
    real_ip::resolve_client_ip,
    request_admission::hold_request_permit,
    request_body_timeout::RequestBodyTimeout,
    request_gate::RequestAdmissionRejection,
    request_uri::{validate_request_uri, RequestUriError},
    static_files::serve_static,
    ListenerState,
};

pub async fn route_request(
    ConnectInfo(connection): ConnectInfo<DownstreamConnectionInfo>,
    State(state): State<ListenerState>,
    request: Request<Body>,
) -> Response<Body> {
    let peer = connection.client_peer;
    let _transport_peer = connection.transport_peer;
    let _proxy_protocol = connection.proxy_protocol;
    let version = request.version();
    let mut admitted = match state.runtime.request_gate.try_begin() {
        Ok(admitted) => admitted,
        Err(RequestAdmissionRejection::Saturated) => {
            state
                .runtime
                .metrics
                .record_request_rejection(RequestRejection::Capacity);
            return overload_response(version);
        }
        Err(RequestAdmissionRejection::ResourcePressure) => {
            state
                .runtime
                .metrics
                .record_request_rejection(RequestRejection::ResourcePressure);
            return resource_pressure_response(version);
        }
    };
    let response_body_idle_timeout = Duration::from_millis(
        state
            .runtime
            .current()
            .app
            .config()
            .limits
            .response_body_idle_timeout_ms,
    );
    let response = route_admitted_request(peer, state, request, &mut admitted).await;
    hold_request_permit(response, admitted, response_body_idle_timeout)
}

async fn route_admitted_request(
    peer: SocketAddr,
    state: ListenerState,
    request: Request<Body>,
    admitted: &mut super::request_gate::RequestAdmissionPermit,
) -> Response<Body> {
    if let Err((status, message)) = validate_request_framing(request.headers(), request.version()) {
        if let Some(response) = classify_request(&state, admitted, false, request.version()) {
            return response;
        }
        return text_response(status, message);
    }
    let generation = state.runtime.current();
    let Some(listener) = generation.app.listener(&state.listener_id) else {
        return text_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "listener configuration is unavailable\n",
        );
    };
    let client_ip = match resolve_client_ip(
        peer.ip(),
        request.headers(),
        listener.trusted_proxy.as_ref(),
    ) {
        Ok(client_ip) => client_ip,
        Err(_) => {
            if let Some(response) = classify_request(&state, admitted, false, request.version()) {
                return response;
            }
            return invalid_forwarded_identity_response(request.version());
        }
    };
    let normalized_path = match validate_request_uri(request.uri(), &generation.app.config().limits)
    {
        Ok(path) => path,
        Err(error) => {
            if let Some(response) = classify_request(&state, admitted, false, request.version()) {
                return response;
            }
            return request_uri_error_response(request.version(), error);
        }
    };
    if content_length_exceeds(
        request.headers(),
        generation.app.config().limits.max_request_body_bytes,
    ) {
        if let Some(response) = classify_request(&state, admitted, false, request.version()) {
            return response;
        }
        return text_response(StatusCode::PAYLOAD_TOO_LARGE, "request body is too large\n");
    }
    let authority = match request_authority(&request) {
        Ok(authority) => authority,
        Err((status, message)) => {
            if let Some(response) = classify_request(&state, admitted, false, request.version()) {
                return response;
            }
            return text_response(status, message);
        }
    };
    let method = request.method().as_str().to_owned();
    let path = normalized_path;
    let Some(selected) =
        generation
            .app
            .select_route(&state.listener_id, &authority, &path, &method)
    else {
        if let Some(response) = classify_request(&state, admitted, false, request.version()) {
            return response;
        }
        return text_response(StatusCode::NOT_FOUND, "route was not found\n");
    };
    let operations_reserved = is_operations_candidate(&request)
        && selected.route.route_match.path_type == RoutePathType::Exact
        && is_operations_path(&selected.route.route_match.path)
        && matches!(selected.resource, ResourceConfig::Respond { .. });
    if let Some(response) =
        classify_request(&state, admitted, operations_reserved, request.version())
    {
        return response;
    }

    let request_failure = RequestBodyFailure::default();
    let (parts, body) = request.into_parts();
    let limits = &generation.app.config().limits;
    let body = RequestBodyTimeout::new_observed(
        body,
        Duration::from_millis(limits.request_body_start_timeout_ms),
        Duration::from_millis(limits.request_body_idle_timeout_ms),
        request_failure.clone(),
        state.runtime.metrics.clone(),
    );
    let request = Request::from_parts(parts, Body::new(body));

    let virtual_host_id = selected.virtual_host.id.clone();
    let route_id = selected.route.id.clone();
    let response = match selected.resource {
        ResourceConfig::Respond {
            status,
            content_type,
            body,
            ..
        } => match drain_bounded_request_body(
            request,
            generation.app.config().limits.max_request_body_bytes,
            &request_failure,
        )
        .await
        {
            Ok(_) => fixed_response(*status, content_type, body, method == "HEAD"),
            Err(response) => response,
        },
        ResourceConfig::Redirect {
            status, location, ..
        } => match drain_bounded_request_body(
            request,
            generation.app.config().limits.max_request_body_bytes,
            &request_failure,
        )
        .await
        {
            Ok(_) => redirect_response(*status, location),
            Err(response) => response,
        },
        ResourceConfig::Static {
            id, spa_fallback, ..
        } => {
            let Some(root) = generation.app.static_root(id) else {
                return text_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "static resource is unavailable\n",
                );
            };
            match drain_bounded_request_body(
                request,
                generation.app.config().limits.max_request_body_bytes,
                &request_failure,
            )
            .await
            {
                Ok(request) => {
                    serve_static(
                        root,
                        selected.route,
                        spa_fallback.as_deref(),
                        &path,
                        request,
                    )
                    .await
                }
                Err(response) => response,
            }
        }
        ResourceConfig::Proxy {
            upstream_ref,
            strip_prefix,
            ..
        } => {
            let scheme = if state.is_tls { "https" } else { "http" };
            proxy_request(
                super::proxy::ProxyRequestContext {
                    generation: &generation,
                    upstream_ref,
                    strip_prefix: *strip_prefix,
                    route: selected.route,
                    client_ip,
                    external_scheme: scheme,
                    external_authority: &authority,
                    normalized_path: &path,
                    request_failure,
                    tunnel_supervisor: &state.runtime.tunnel_supervisor,
                    metrics: &state.runtime.metrics,
                },
                request,
            )
            .await
        }
    };

    if generation.app.config().observability.access_log {
        tracing::info!(
            config_generation = generation.id,
            config_revision = %generation.revision,
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

fn invalid_forwarded_identity_response(version: Version) -> Response<Body> {
    let mut response = text_response(
        StatusCode::BAD_REQUEST,
        "forwarded client identity is invalid\n",
    );
    if version != Version::HTTP_2 {
        response
            .headers_mut()
            .insert(CONNECTION, HeaderValue::from_static("close"));
    }
    response
}

fn classify_request(
    state: &ListenerState,
    admitted: &mut super::request_gate::RequestAdmissionPermit,
    operations_reserved: bool,
    version: Version,
) -> Option<Response<Body>> {
    state
        .runtime
        .request_gate
        .classify(admitted, operations_reserved)
        .err()
        .map(|rejection| match rejection {
            RequestAdmissionRejection::Saturated => {
                state
                    .runtime
                    .metrics
                    .record_request_rejection(RequestRejection::Capacity);
                overload_response(version)
            }
            RequestAdmissionRejection::ResourcePressure => {
                state
                    .runtime
                    .metrics
                    .record_request_rejection(RequestRejection::ResourcePressure);
                resource_pressure_response(version)
            }
        })
}

fn overload_response(version: Version) -> Response<Body> {
    let mut response = text_response(StatusCode::SERVICE_UNAVAILABLE, "server is overloaded\n");
    response
        .headers_mut()
        .insert(RETRY_AFTER, HeaderValue::from_static("1"));
    if version != Version::HTTP_2 {
        response
            .headers_mut()
            .insert(CONNECTION, HeaderValue::from_static("close"));
    }
    response
}

fn resource_pressure_response(version: Version) -> Response<Body> {
    let mut response = text_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "server resource pressure is active\n",
    );
    response
        .headers_mut()
        .insert(RETRY_AFTER, HeaderValue::from_static("1"));
    if version != Version::HTTP_2 {
        response
            .headers_mut()
            .insert(CONNECTION, HeaderValue::from_static("close"));
    }
    response
}

fn is_operations_candidate(request: &Request<Body>) -> bool {
    matches!(request.method().as_str(), "GET" | "HEAD") && is_operations_path(request.uri().path())
}

fn is_operations_path(path: &str) -> bool {
    matches!(path, "/healthz" | "/readyz" | "/livez")
}

fn request_uri_error_response(version: Version, error: RequestUriError) -> Response<Body> {
    let (status, message) = match error {
        RequestUriError::Invalid => (StatusCode::BAD_REQUEST, "request URI is invalid\n"),
        RequestUriError::TooLong => (StatusCode::URI_TOO_LONG, "request URI exceeds limits\n"),
    };
    let mut response = text_response(status, message);
    if version != Version::HTTP_2 {
        response
            .headers_mut()
            .insert(CONNECTION, HeaderValue::from_static("close"));
    }
    response
}

async fn drain_bounded_request_body(
    request: Request<Body>,
    maximum: u64,
    failure: &RequestBodyFailure,
) -> Result<Request<Body>, Response<Body>> {
    let (parts, body) = request.into_parts();
    let version = parts.version;
    let mut stream = body.into_data_stream();
    let mut observed = 0_u64;
    while let Some(frame) = stream.next().await {
        let bytes = frame.map_err(|_| {
            if failure.timed_out() {
                request_body_timeout_response(version)
            } else {
                text_response(StatusCode::BAD_REQUEST, "request body framing is invalid\n")
            }
        })?;
        observed = observed.saturating_add(bytes.len() as u64);
        if observed > maximum {
            return Err(text_response(
                StatusCode::PAYLOAD_TOO_LARGE,
                "request body is too large\n",
            ));
        }
    }
    Ok(Request::from_parts(parts, Body::empty()))
}

fn content_length_exceeds(headers: &HeaderMap, maximum: u64) -> bool {
    headers
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .is_some_and(|length| length > maximum)
}

fn validate_request_framing(
    headers: &HeaderMap,
    version: Version,
) -> Result<(), (StatusCode, &'static str)> {
    validate_expectation(headers, version)?;

    let mut content_lengths = headers.get_all(CONTENT_LENGTH).iter();
    let has_content_length = content_lengths.next().is_some();
    if content_lengths.next().is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            "multiple Content-Length headers are forbidden\n",
        ));
    }
    let has_transfer_encoding = headers.contains_key(TRANSFER_ENCODING);
    if has_content_length && has_transfer_encoding {
        return Err((
            StatusCode::BAD_REQUEST,
            "Transfer-Encoding with Content-Length is forbidden\n",
        ));
    }
    if version != Version::HTTP_11 && has_transfer_encoding {
        return Err((
            StatusCode::BAD_REQUEST,
            "Transfer-Encoding requires HTTP/1.1\n",
        ));
    }
    for value in headers.get_all(TE) {
        let value = value
            .to_str()
            .map_err(|_| (StatusCode::BAD_REQUEST, "invalid TE header\n"))?;
        if value
            .split(',')
            .map(str::trim)
            .any(|token| !token.eq_ignore_ascii_case("trailers"))
        {
            return Err((StatusCode::BAD_REQUEST, "only TE: trailers is supported\n"));
        }
    }
    Ok(())
}

fn validate_expectation(
    headers: &HeaderMap,
    version: Version,
) -> Result<(), (StatusCode, &'static str)> {
    let mut expectations = headers.get_all(EXPECT).iter();
    let Some(expectation) = expectations.next() else {
        return Ok(());
    };
    if expectations.next().is_some()
        || version != Version::HTTP_11
        || expectation
            .to_str()
            .map(|value| !value.eq_ignore_ascii_case("100-continue"))
            .unwrap_or(true)
    {
        return Err((
            StatusCode::EXPECTATION_FAILED,
            "request expectation is not supported\n",
        ));
    }
    Ok(())
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
