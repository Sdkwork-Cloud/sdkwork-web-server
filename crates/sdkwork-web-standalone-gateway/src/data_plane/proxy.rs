use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use axum::{
    body::Body,
    http::{
        header::{
            CONNECTION, CONTENT_LENGTH, EXPECT, HOST, PROXY_AUTHENTICATE, PROXY_AUTHORIZATION, TE,
            TRANSFER_ENCODING, UPGRADE,
        },
        HeaderMap, HeaderName, HeaderValue, Request, Response, StatusCode, Version,
    },
};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use reqwest::{redirect::Policy, Client, Url};
use sdkwork_webserver_core::{CompiledWebServerApp, RouteConfig, UpstreamConfig};

use super::{
    dns::{BoundedSystemResolver, GuardedDnsResolver},
    proxy_body::{
        validate_trailer_declaration, GuardedProxyBody, ProxyRequestBodyControl,
        ProxyTrailerPolicy, RequestBodyFailure,
    },
    runtime::RuntimeGeneration,
    upstream_tls::configure_upstream_tls,
    DataPlaneError,
};

const CANONICAL_PROXY_PATH_ENCODE_SET: &AsciiSet = &CONTROLS.add(b'%');

pub struct ProxyUpstream {
    client: Client,
    targets: Vec<Url>,
    cursor: AtomicUsize,
}

impl ProxyUpstream {
    pub(crate) fn build(
        app: &CompiledWebServerApp,
        config: &UpstreamConfig,
        resolver: Arc<BoundedSystemResolver>,
    ) -> Result<Self, DataPlaneError> {
        let resolver = Arc::new(GuardedDnsResolver::new(
            resolver,
            config.address_policy.clone(),
        ));
        let builder = Client::builder()
            .redirect(Policy::none())
            .connect_timeout(Duration::from_millis(config.connect_timeout_ms))
            .timeout(Duration::from_millis(config.request_timeout_ms))
            .pool_max_idle_per_host(config.max_idle_connections)
            .pool_idle_timeout(Duration::from_millis(config.idle_connection_timeout_ms))
            .dns_resolver(resolver);
        let client = configure_upstream_tls(builder, app, config)?
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

pub(super) struct ProxyRequestContext<'a> {
    pub generation: &'a RuntimeGeneration,
    pub upstream_ref: &'a str,
    pub strip_prefix: bool,
    pub route: &'a RouteConfig,
    pub peer: SocketAddr,
    pub external_scheme: &'a str,
    pub external_authority: &'a str,
    pub normalized_path: &'a str,
    pub request_failure: RequestBodyFailure,
}

pub(super) async fn proxy_request(
    context: ProxyRequestContext<'_>,
    request: Request<Body>,
) -> Response<Body> {
    let Some(upstream) = context.generation.upstreams.get(context.upstream_ref) else {
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
        context.normalized_path,
        request.uri().query(),
    ) {
        Ok(url) => url,
        Err(()) => return text_response(StatusCode::BAD_GATEWAY, "invalid upstream target\n"),
    };

    let request_version = request.version();
    let (parts, body) = request.into_parts();
    let maximum_body_bytes = context
        .generation
        .app
        .config()
        .limits
        .max_request_body_bytes;
    let maximum_trailer_bytes = context.generation.app.config().limits.max_trailer_bytes;
    let maximum_trailers = context.generation.app.config().limits.max_trailers;
    let (headers, forbidden_request_trailers, declared_request_trailers) =
        match forwarded_request_headers(
            &parts.headers,
            context.peer,
            context.external_scheme,
            context.external_authority,
            maximum_trailer_bytes,
            maximum_trailers,
        ) {
            Ok(result) => result,
            Err(()) => {
                return text_response(StatusCode::BAD_REQUEST, "invalid Trailer declaration\n")
            }
        };
    let request_control = ProxyRequestBodyControl::default();
    let guarded_body = GuardedProxyBody::request(
        body,
        maximum_body_bytes,
        ProxyTrailerPolicy::new(
            maximum_trailer_bytes,
            maximum_trailers,
            declared_request_trailers,
            forbidden_request_trailers,
        ),
        context.request_failure.clone(),
        request_control.clone(),
    );

    let result = upstream
        .client
        .request(parts.method, target_url)
        .headers(headers)
        .body(reqwest::Body::wrap(guarded_body))
        .send()
        .await;
    let response = match result {
        Ok(response) => response,
        Err(_) if context.request_failure.timed_out() => {
            return request_body_timeout_response(request_version)
        }
        Err(_) if context.request_failure.body_too_large() => {
            return text_response(StatusCode::PAYLOAD_TOO_LARGE, "request body is too large\n")
        }
        Err(_) if context.request_failure.invalid_body() => {
            return text_response(StatusCode::BAD_REQUEST, "request body framing is invalid\n")
        }
        Err(error) if error.is_timeout() => {
            return text_response(StatusCode::GATEWAY_TIMEOUT, "upstream timed out\n")
        }
        Err(_) => return text_response(StatusCode::BAD_GATEWAY, "upstream failed\n"),
    };

    let upstream_responded_early = request_control.pause_if_incomplete();
    let response: Response<reqwest::Body> = response.into();
    let (mut parts, body) = response.into_parts();
    let (response_headers, forbidden_response_trailers, declared_response_trailers) =
        match forwarded_response_headers(&parts.headers, maximum_trailer_bytes, maximum_trailers) {
            Ok(result) => result,
            Err(()) => {
                request_control.cancel_if_incomplete();
                return text_response(
                    StatusCode::BAD_GATEWAY,
                    "upstream Trailer declaration is invalid\n",
                );
            }
        };
    parts.headers = response_headers;
    if upstream_responded_early && request_version != Version::HTTP_2 {
        parts
            .headers
            .insert(CONNECTION, HeaderValue::from_static("close"));
    }
    let response_trailer_policy = ProxyTrailerPolicy::new(
        maximum_trailer_bytes,
        maximum_trailers,
        declared_response_trailers,
        forbidden_response_trailers,
    );
    let guarded_body = if upstream_responded_early {
        GuardedProxyBody::response_with_request_cancellation(
            body,
            response_trailer_policy,
            request_control,
        )
    } else {
        GuardedProxyBody::response(body, response_trailer_policy)
    };
    Response::from_parts(parts, Body::new(guarded_body))
}

fn build_target_url(
    target: &Url,
    strip_prefix: bool,
    route_path: &str,
    request_path: &str,
    normalized_path: &str,
    query: Option<&str>,
) -> Result<Url, ()> {
    if strip_prefix {
        let forwarded_path = normalized_path
            .strip_prefix(route_path)
            .unwrap_or(normalized_path);
        let mut rewritten = target.clone();
        let base_path = rewritten.path().trim_end_matches('/');
        let path = if forwarded_path.is_empty() {
            "/"
        } else {
            forwarded_path
        };
        let combined_path = format!("{base_path}/{}", path.trim_start_matches('/'));
        let encoded_path =
            utf8_percent_encode(&combined_path, CANONICAL_PROXY_PATH_ENCODE_SET).to_string();
        rewritten.set_path(&encoded_path);
        rewritten.set_query(query);
        return Ok(rewritten);
    }
    let forwarded_path = request_path;
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
    maximum_trailer_bytes: usize,
    maximum_trailers: usize,
) -> Result<(HeaderMap, HashSet<HeaderName>, HashSet<HeaderName>), ()> {
    let hop_by_hop = hop_by_hop_headers(source);
    let declared_trailers =
        validate_trailer_declaration(source, maximum_trailer_bytes, maximum_trailers, &hop_by_hop)?;
    let mut target = HeaderMap::new();
    for (name, value) in source {
        if name != HOST && name != CONTENT_LENGTH && name != EXPECT && !hop_by_hop.contains(name) {
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
    target.insert(TE, HeaderValue::from_static("trailers"));
    Ok((target, hop_by_hop, declared_trailers))
}

fn forwarded_response_headers(
    source: &HeaderMap,
    maximum_trailer_bytes: usize,
    maximum_trailers: usize,
) -> Result<(HeaderMap, HashSet<HeaderName>, HashSet<HeaderName>), ()> {
    let hop_by_hop = hop_by_hop_headers(source);
    let declared_trailers =
        validate_trailer_declaration(source, maximum_trailer_bytes, maximum_trailers, &hop_by_hop)?;
    let mut target = HeaderMap::new();
    for (name, value) in source {
        if !hop_by_hop.contains(name) {
            target.append(name.clone(), value.clone());
        }
    }
    Ok((target, hop_by_hop, declared_trailers))
}

fn hop_by_hop_headers(headers: &HeaderMap) -> HashSet<HeaderName> {
    let mut names = HashSet::from([
        CONNECTION,
        HeaderName::from_static("keep-alive"),
        PROXY_AUTHENTICATE,
        PROXY_AUTHORIZATION,
        TE,
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

pub(crate) fn text_response(status: StatusCode, body: &'static str) -> Response<Body> {
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status;
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    response
}

pub(super) fn request_body_timeout_response(version: Version) -> Response<Body> {
    let mut response = text_response(
        StatusCode::REQUEST_TIMEOUT,
        "request Body progress timed out\n",
    );
    if version != Version::HTTP_2 {
        response
            .headers_mut()
            .insert(CONNECTION, HeaderValue::from_static("close"));
    }
    response
}

#[cfg(test)]
mod tests {
    use super::{build_target_url, Url};

    #[test]
    fn rewritten_target_encodes_canonical_reserved_and_unicode_path_bytes() {
        let target = Url::parse("https://origin.example/base").expect("valid target");
        let rewritten = build_target_url(
            &target,
            true,
            "/rewrite",
            "/rewrite/a%3Fb%23c%25d/%E4%B8%AD",
            "/rewrite/a?b#c%d/中",
            Some("x=%2F&y=1"),
        )
        .expect("rewritten URL");

        assert_eq!(
            rewritten.as_str(),
            "https://origin.example/base/a%3Fb%23c%25d/%E4%B8%AD?x=%2F&y=1"
        );
        assert_eq!(rewritten.path(), "/base/a%3Fb%23c%25d/%E4%B8%AD");
        assert_eq!(rewritten.query(), Some("x=%2F&y=1"));
        assert_eq!(rewritten.fragment(), None);
    }

    #[test]
    fn no_rewrite_target_preserves_raw_path_and_query_encoding() {
        let target = Url::parse("https://origin.example/base").expect("valid target");
        let rewritten = build_target_url(
            &target,
            false,
            "/rewrite",
            "/rewrite/a%2Fb/%E4%B8%AD",
            "/rewrite/a/b/中",
            Some("x=%2F&y=1"),
        )
        .expect("raw URL");

        assert_eq!(
            rewritten.as_str(),
            "https://origin.example/base/rewrite/a%2Fb/%E4%B8%AD?x=%2F&y=1"
        );
    }
}
