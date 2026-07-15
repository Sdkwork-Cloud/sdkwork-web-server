use std::{
    collections::HashSet,
    io,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use axum::{
    body::Body,
    http::{
        header::{
            CONNECTION, CONTENT_LENGTH, HOST, PROXY_AUTHENTICATE, PROXY_AUTHORIZATION, TE, TRAILER,
            TRANSFER_ENCODING, UPGRADE,
        },
        HeaderMap, HeaderName, HeaderValue, Request, Response, StatusCode,
    },
};
use futures_util::StreamExt;
use reqwest::{redirect::Policy, Client, Url};
use sdkwork_webserver_core::{RouteConfig, UpstreamConfig};

use super::{DataPlaneError, DataPlaneRuntime};

pub struct ProxyUpstream {
    client: Client,
    targets: Vec<Url>,
    cursor: AtomicUsize,
}

impl ProxyUpstream {
    pub fn build(config: &UpstreamConfig) -> Result<Self, DataPlaneError> {
        let client = Client::builder()
            .redirect(Policy::none())
            .connect_timeout(Duration::from_millis(config.connect_timeout_ms))
            .timeout(Duration::from_millis(config.request_timeout_ms))
            .pool_max_idle_per_host(config.max_idle_connections)
            .build()
            .map_err(|source| DataPlaneError::UpstreamClient {
                upstream_id: config.id.clone(),
                source,
            })?;
        let targets = config
            .targets
            .iter()
            .map(|target| {
                Url::parse(&target.url).map_err(|_| DataPlaneError::InvalidUpstreamTarget {
                    upstream_id: config.id.clone(),
                    target: target.url.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            client,
            targets,
            cursor: AtomicUsize::new(0),
        })
    }

    fn next_target(&self) -> &Url {
        let index = self.cursor.fetch_add(1, Ordering::Relaxed) % self.targets.len();
        &self.targets[index]
    }
}

pub struct ProxyRequestContext<'a> {
    pub runtime: &'a DataPlaneRuntime,
    pub upstream_ref: &'a str,
    pub strip_prefix: bool,
    pub route: &'a RouteConfig,
    pub peer: SocketAddr,
    pub external_scheme: &'a str,
    pub external_authority: &'a str,
}

pub async fn proxy_request(
    context: ProxyRequestContext<'_>,
    request: Request<Body>,
) -> Response<Body> {
    let Some(upstream) = context.runtime.upstreams.get(context.upstream_ref) else {
        return text_response(StatusCode::BAD_GATEWAY, "upstream is unavailable\n");
    };
    if request.headers().contains_key(UPGRADE) {
        return text_response(
            StatusCode::NOT_IMPLEMENTED,
            "protocol upgrade is not implemented by REQ-2026-0003\n",
        );
    }

    let target_url = match build_target_url(
        upstream.next_target(),
        context.strip_prefix,
        &context.route.route_match.path,
        request.uri().path(),
        request.uri().query(),
    ) {
        Ok(url) => url,
        Err(()) => return text_response(StatusCode::BAD_GATEWAY, "invalid upstream target\n"),
    };

    let (parts, body) = request.into_parts();
    let maximum_body_bytes = context.runtime.app.config().limits.max_request_body_bytes;
    if content_length_exceeds(&parts.headers, maximum_body_bytes) {
        return text_response(StatusCode::PAYLOAD_TOO_LARGE, "request body is too large\n");
    }

    let headers = forwarded_request_headers(
        &parts.headers,
        context.peer,
        context.external_scheme,
        context.external_authority,
    );
    let exceeded = Arc::new(AtomicBool::new(false));
    let observed = Arc::new(AtomicU64::new(0));
    let exceeded_for_stream = exceeded.clone();
    let observed_for_stream = observed.clone();
    let body_stream = body.into_data_stream().map(move |result| {
        let bytes = result.map_err(io::Error::other)?;
        let length = bytes.len() as u64;
        let previous = observed_for_stream.fetch_add(length, Ordering::Relaxed);
        if previous.saturating_add(length) > maximum_body_bytes {
            exceeded_for_stream.store(true, Ordering::Relaxed);
            Err(io::Error::other("request body limit exceeded"))
        } else {
            Ok(bytes)
        }
    });

    let result = upstream
        .client
        .request(parts.method, target_url)
        .headers(headers)
        .body(reqwest::Body::wrap_stream(body_stream))
        .send()
        .await;
    let response = match result {
        Ok(response) => response,
        Err(_) if exceeded.load(Ordering::Relaxed) => {
            return text_response(StatusCode::PAYLOAD_TOO_LARGE, "request body is too large\n")
        }
        Err(error) if error.is_timeout() => {
            return text_response(StatusCode::GATEWAY_TIMEOUT, "upstream timed out\n")
        }
        Err(_) => return text_response(StatusCode::BAD_GATEWAY, "upstream failed\n"),
    };

    let status = response.status();
    let response_headers = forwarded_response_headers(response.headers());
    let response_stream = response
        .bytes_stream()
        .map(|result| result.map_err(io::Error::other));
    let mut output = Response::new(Body::from_stream(response_stream));
    *output.status_mut() = status;
    *output.headers_mut() = response_headers;
    output
}

fn build_target_url(
    target: &Url,
    strip_prefix: bool,
    route_path: &str,
    request_path: &str,
    query: Option<&str>,
) -> Result<Url, ()> {
    let forwarded_path = if strip_prefix {
        request_path
            .strip_prefix(route_path)
            .unwrap_or(request_path)
    } else {
        request_path
    };
    let base = target.as_str().trim_end_matches('/');
    let path = if forwarded_path.is_empty() {
        "/"
    } else {
        forwarded_path
    };
    let mut combined = format!("{base}/{}", path.trim_start_matches('/'));
    if let Some(query) = query {
        combined.push('?');
        combined.push_str(query);
    }
    Url::parse(&combined).map_err(|_| ())
}

fn forwarded_request_headers(
    source: &HeaderMap,
    peer: SocketAddr,
    external_scheme: &str,
    external_authority: &str,
) -> HeaderMap {
    let hop_by_hop = hop_by_hop_headers(source);
    let mut target = HeaderMap::new();
    for (name, value) in source {
        if name != HOST && !hop_by_hop.contains(name) {
            target.append(name.clone(), value.clone());
        }
    }
    if let Ok(value) = HeaderValue::from_str(&peer.ip().to_string()) {
        target.insert(HeaderName::from_static("x-forwarded-for"), value);
    }
    if let Ok(value) = HeaderValue::from_str(external_scheme) {
        target.insert(HeaderName::from_static("x-forwarded-proto"), value);
    }
    if let Ok(value) = HeaderValue::from_str(external_authority) {
        target.insert(HeaderName::from_static("x-forwarded-host"), value);
    }
    target
}

fn forwarded_response_headers(source: &HeaderMap) -> HeaderMap {
    let hop_by_hop = hop_by_hop_headers(source);
    let mut target = HeaderMap::new();
    for (name, value) in source {
        if !hop_by_hop.contains(name) {
            target.append(name.clone(), value.clone());
        }
    }
    target
}

fn hop_by_hop_headers(headers: &HeaderMap) -> HashSet<HeaderName> {
    let mut names = HashSet::from([
        CONNECTION,
        HeaderName::from_static("keep-alive"),
        PROXY_AUTHENTICATE,
        PROXY_AUTHORIZATION,
        TE,
        TRAILER,
        TRANSFER_ENCODING,
        UPGRADE,
        HeaderName::from_static("proxy-connection"),
    ]);
    for value in headers.get_all(CONNECTION) {
        if let Ok(value) = value.to_str() {
            for token in value.split(',').map(str::trim) {
                if let Ok(name) = HeaderName::from_bytes(token.as_bytes()) {
                    names.insert(name);
                }
            }
        }
    }
    names
}

fn content_length_exceeds(headers: &HeaderMap, maximum: u64) -> bool {
    headers
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .is_some_and(|length| length > maximum)
}

pub(crate) fn text_response(status: StatusCode, body: &'static str) -> Response<Body> {
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status;
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    response
}
