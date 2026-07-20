use std::{
    collections::HashSet,
    net::IpAddr,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use axum::{
    body::Body,
    http::{
        header::{
            CONNECTION, CONTENT_LENGTH, EXPECT, HOST, PROXY_AUTHENTICATE, PROXY_AUTHORIZATION,
            RETRY_AFTER, TE, TRANSFER_ENCODING, UPGRADE,
        },
        HeaderMap, HeaderName, HeaderValue, Method, Request, Response, StatusCode, Version,
    },
};
use http_body::Body as HttpBody;
use http_body_util::BodyExt;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use sdkwork_webserver_core::{
    CompiledWebServerApp, RouteConfig, UpstreamActiveHealthConfig, UpstreamActiveHealthMethod,
    UpstreamConfig, UpstreamLoadBalancingStrategy, UpstreamRetryCondition,
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use url::Url;

use super::{
    dns::{BoundedSystemResolver, GuardedDnsResolver},
    http1_wire::Http1UpgradeGuard,
    metrics::{
        DataPlaneMetrics, UpstreamMetricLease, UpstreamRejection, UpstreamResult,
        UpstreamRetryReason,
    },
    proxy_body::{
        validate_trailer_declaration, GuardedProxyBody, ProxyRequestBodyControl,
        ProxyTrailerPolicy, RequestBodyFailure,
    },
    runtime::RuntimeGeneration,
    smooth_weighted::SmoothWeightedState,
    tunnel::TunnelSupervisor,
    upstream_admission::hold_upstream_permit,
    upstream_client::{UpstreamClient, UpstreamResponseBody},
    DataPlaneError,
};

const CANONICAL_PROXY_PATH_ENCODE_SET: &AsciiSet = &CONTROLS.add(b'%');
const ATTEMPTED_TARGET_WORDS: usize = 16;
const SPLITMIX64_GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;
static RANDOM_SEED_SEQUENCE: AtomicU64 = AtomicU64::new(SPLITMIX64_GAMMA);

pub struct ProxyUpstream<T = UpstreamClient> {
    id: String,
    client: T,
    targets: Vec<ProxyTarget>,
    load_balancing: UpstreamLoadBalancingStrategy,
    smooth_weighted: SmoothWeightedState,
    cursor: AtomicUsize,
    random_state: AtomicU64,
    permits: Arc<Semaphore>,
    max_in_flight_requests: usize,
    retry: RetryPolicy,
    health: PassiveHealthPolicy,
    active_health: Option<ActiveHealthPolicy>,
    epoch: Instant,
}

pub(super) struct ProxyTarget {
    url: Url,
    weight: usize,
    pub(super) backup: bool,
    slow_start_duration_ms: u64,
    slow_start_started_ms: AtomicU64,
    active_requests: Arc<AtomicUsize>,
    consecutive_failures: AtomicU32,
    ejected_until_ms: AtomicU64,
    probe_in_flight: AtomicBool,
    active_available: AtomicBool,
    active_failures: AtomicU32,
    active_successes: AtomicU32,
    active_health_url: Option<Url>,
}

struct PassiveHealthPolicy {
    failure_threshold: u32,
    ejection_time_ms: u64,
    failure_statuses: Vec<u16>,
}

#[derive(Clone, Copy)]
struct RetryPolicy {
    enabled: bool,
    maximum_attempts: usize,
    total_timeout: Duration,
    attempt_timeout: Duration,
    transport_failure: bool,
    timeout: bool,
    statuses: [bool; 3],
}

#[derive(Clone, Copy)]
struct RetryTargetContext<'a> {
    client_ip: IpAddr,
    attempts_started: usize,
    maximum_attempts: usize,
    deadline: Instant,
    metrics: &'a DataPlaneMetrics,
    reason: UpstreamRetryReason,
}

impl RetryPolicy {
    fn from_config(config: &UpstreamConfig) -> Self {
        let mut policy = Self {
            enabled: false,
            maximum_attempts: 1,
            total_timeout: Duration::from_millis(config.request_timeout_ms),
            attempt_timeout: Duration::from_millis(config.request_timeout_ms),
            transport_failure: false,
            timeout: false,
            statuses: [false; 3],
        };
        let Some(retry) = &config.retry else {
            return policy;
        };
        policy.enabled = true;
        policy.maximum_attempts = usize::from(retry.max_attempts);
        policy.total_timeout = Duration::from_millis(retry.timeout_ms);
        for condition in &retry.retry_on {
            match condition {
                UpstreamRetryCondition::TransportFailure => policy.transport_failure = true,
                UpstreamRetryCondition::Timeout => policy.timeout = true,
                UpstreamRetryCondition::Http502 => policy.statuses[0] = true,
                UpstreamRetryCondition::Http503 => policy.statuses[1] = true,
                UpstreamRetryCondition::Http504 => policy.statuses[2] = true,
            }
        }
        policy
    }

    fn status_reason(self, status: StatusCode) -> Option<UpstreamRetryReason> {
        match status.as_u16() {
            502 if self.statuses[0] => Some(UpstreamRetryReason::Http502),
            503 if self.statuses[1] => Some(UpstreamRetryReason::Http503),
            504 if self.statuses[2] => Some(UpstreamRetryReason::Http504),
            _ => None,
        }
    }
}

#[derive(Default)]
pub(super) struct AttemptedTargets {
    words: [u64; ATTEMPTED_TARGET_WORDS],
}

impl AttemptedTargets {
    pub(super) fn contains(&self, index: usize) -> bool {
        let word = index / u64::BITS as usize;
        let bit = index % u64::BITS as usize;
        self.words
            .get(word)
            .is_some_and(|value| value & (1_u64 << bit) != 0)
    }

    fn insert(&mut self, index: usize) {
        let word = index / u64::BITS as usize;
        let bit = index % u64::BITS as usize;
        if let Some(value) = self.words.get_mut(word) {
            *value |= 1_u64 << bit;
        }
    }
}

struct ActiveHealthPolicy {
    method: UpstreamActiveHealthMethod,
    interval: Duration,
    timeout: Duration,
    unhealthy_threshold: u32,
    healthy_threshold: u32,
    success_status_min: u16,
    success_status_max: u16,
    max_response_body_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ActiveHealthTransition {
    Unchanged,
    BecameHealthy,
    BecameUnhealthy,
}

impl ActiveHealthTransition {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Unchanged => "unchanged",
            Self::BecameHealthy => "healthy",
            Self::BecameUnhealthy => "unhealthy",
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct SelectedTarget<'a> {
    index: usize,
    url: &'a Url,
    probe: bool,
    ejection_deadline_ms: u64,
}

struct ProbeClaimLease<'a> {
    flag: Option<&'a AtomicBool>,
}

pub(crate) struct TargetActivityLease {
    counter: Arc<AtomicUsize>,
    acquired: bool,
}

impl TargetActivityLease {
    fn claim(counter: &Arc<AtomicUsize>) -> Self {
        let acquired = counter
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current.checked_add(1)
            })
            .is_ok();
        Self {
            counter: counter.clone(),
            acquired,
        }
    }
}

impl Drop for TargetActivityLease {
    fn drop(&mut self) {
        if self.acquired {
            let _ = self
                .counter
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                    current.checked_sub(1)
                });
        }
    }
}

impl Drop for ProbeClaimLease<'_> {
    fn drop(&mut self) {
        if let Some(flag) = self.flag {
            flag.store(false, Ordering::Release);
        }
    }
}

pub(super) struct ProxyRequestContext<'a> {
    pub generation: &'a Arc<RuntimeGeneration>,
    pub upstream_ref: &'a str,
    pub strip_prefix: bool,
    pub route: &'a RouteConfig,
    pub client_ip: IpAddr,
    pub external_scheme: &'a str,
    pub external_authority: &'a str,
    pub normalized_path: &'a str,
    pub request_failure: RequestBodyFailure,
    pub tunnel_supervisor: &'a Arc<TunnelSupervisor>,
    pub metrics: &'a Arc<DataPlaneMetrics>,
}

mod upstream;
#[cfg(test)]
use upstream::{advance_ip_hash, weighted_load_cmp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpgradeDisposition {
    None,
    WebSocket,
    Unsupported,
}

pub(super) async fn proxy_request(
    context: ProxyRequestContext<'_>,
    request: Request<Body>,
) -> Response<Body> {
    let upgrade = match classify_upgrade_request(&request) {
        Ok(upgrade) => upgrade,
        Err(()) => return text_response(StatusCode::BAD_REQUEST, "invalid protocol upgrade\n"),
    };
    if upgrade == UpgradeDisposition::Unsupported {
        return text_response(
            StatusCode::NOT_IMPLEMENTED,
            "protocol upgrade is unsupported\n",
        );
    }
    if upgrade == UpgradeDisposition::WebSocket {
        if let Some(guard) = request.extensions().get::<Http1UpgradeGuard>() {
            guard.activate();
        }
    }
    let Some(upstream) = context.generation.upstreams.get(context.upstream_ref) else {
        context
            .metrics
            .record_upstream_rejection(UpstreamRejection::MissingUpstream);
        return upgrade_failure_response(
            upgrade,
            text_response(StatusCode::BAD_GATEWAY, "upstream is unavailable\n"),
        );
    };

    let upstream_permit = match upstream.try_admit() {
        Ok(permit) => permit,
        Err(()) => {
            context
                .metrics
                .record_upstream_rejection(UpstreamRejection::RequestCapacity);
            return upgrade_failure_response(
                upgrade,
                upstream_unavailable_response("upstream is saturated\n"),
            );
        }
    };
    let Some(selected) = upstream.select_target_observed(context.client_ip, Some(context.metrics))
    else {
        context
            .metrics
            .record_upstream_rejection(UpstreamRejection::NoEligibleTarget);
        return upgrade_failure_response(
            upgrade,
            upstream_unavailable_response("all upstream targets are unavailable\n"),
        );
    };
    let target_activity = upstream.claim_target_activity(selected.index);

    let target_url = match build_target_url(
        selected.url,
        context.strip_prefix,
        &context.route.route_match.path,
        request.uri().path(),
        context.normalized_path,
        request.uri().query(),
    ) {
        Ok(url) => url,
        Err(()) => {
            upstream.abandon_probe(selected);
            return upgrade_failure_response(
                upgrade,
                text_response(StatusCode::BAD_GATEWAY, "invalid upstream target\n"),
            );
        }
    };

    if upgrade == UpgradeDisposition::WebSocket {
        return proxy_websocket_request(
            &context,
            upstream,
            selected,
            target_activity,
            upstream_permit,
            target_url,
            request,
        )
        .await;
    }
    proxy_http_request(
        &context,
        upstream,
        selected,
        target_activity,
        target_url,
        upstream_permit,
        request,
    )
    .await
}

async fn proxy_http_request(
    context: &ProxyRequestContext<'_>,
    upstream: &ProxyUpstream,
    mut selected: SelectedTarget<'_>,
    mut target_activity: TargetActivityLease,
    mut target_url: Url,
    upstream_permit: OwnedSemaphorePermit,
    request: Request<Body>,
) -> Response<Body> {
    let request_version = request.version();
    let retryable_request = is_bodyless_idempotent_request(&request);
    let (request_parts, body) = request.into_parts();
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
            &request_parts.headers,
            context.client_ip,
            context.external_scheme,
            context.external_authority,
            maximum_trailer_bytes,
            maximum_trailers,
        ) {
            Ok(result) => result,
            Err(()) => {
                upstream.abandon_probe(selected);
                return text_response(StatusCode::BAD_REQUEST, "invalid Trailer declaration\n");
            }
        };
    let mut request_body = Some(body);
    let mut request_trailer_policy = Some((forbidden_request_trailers, declared_request_trailers));
    let retry_enabled = retryable_request && upstream.retry.enabled;
    let maximum_attempts = if retry_enabled {
        upstream.retry.maximum_attempts
    } else {
        1
    };
    let retry_deadline = Instant::now() + upstream.retry.total_timeout;
    let mut attempted = AttemptedTargets::default();
    attempted.insert(selected.index);
    let mut attempts_started = 0usize;

    loop {
        attempts_started = attempts_started.saturating_add(1);
        let _probe_claim_lease = upstream.probe_claim_lease(selected.index, selected.probe);
        let target_uri = match target_url.as_str().parse() {
            Ok(uri) => uri,
            Err(_) => {
                upstream.abandon_probe(selected);
                return text_response(StatusCode::BAD_GATEWAY, "invalid upstream target\n");
            }
        };
        let request_control = if retryable_request {
            ProxyRequestBodyControl::completed()
        } else {
            ProxyRequestBodyControl::default()
        };
        let upstream_body = if retryable_request {
            Body::empty()
        } else {
            let (forbidden, declared) = request_trailer_policy
                .take()
                .expect("non-replayed request owns one Trailer policy");
            Body::new(GuardedProxyBody::request(
                request_body
                    .take()
                    .expect("non-replayed request owns one Body"),
                maximum_body_bytes,
                ProxyTrailerPolicy::new(
                    maximum_trailer_bytes,
                    maximum_trailers,
                    declared,
                    forbidden,
                ),
                context.request_failure.clone(),
                request_control.clone(),
            ))
        };
        let mut upstream_request = Request::new(upstream_body);
        *upstream_request.method_mut() = request_parts.method.clone();
        *upstream_request.uri_mut() = target_uri;
        *upstream_request.headers_mut() = headers.clone();

        let attempt = context.metrics.begin_upstream_attempt();
        let result = if retry_enabled {
            let remaining = retry_deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                attempt.finish(UpstreamResult::Timeout);
                upstream.abandon_probe(selected);
                return text_response(StatusCode::GATEWAY_TIMEOUT, "upstream timed out\n");
            }
            upstream
                .client
                .execute_with_timeout(
                    upstream_request,
                    upstream.retry.attempt_timeout.min(remaining),
                )
                .await
        } else {
            upstream.client.execute(upstream_request).await
        };
        let response = match result {
            Ok(response) => response,
            Err(_) if context.request_failure.timed_out() => {
                attempt.finish(UpstreamResult::RequestFailure);
                upstream.abandon_probe(selected);
                return request_body_timeout_response(request_version);
            }
            Err(_) if context.request_failure.body_too_large() => {
                attempt.finish(UpstreamResult::RequestFailure);
                upstream.abandon_probe(selected);
                return text_response(StatusCode::PAYLOAD_TOO_LARGE, "request body is too large\n");
            }
            Err(_) if context.request_failure.invalid_body() => {
                attempt.finish(UpstreamResult::RequestFailure);
                upstream.abandon_probe(selected);
                return text_response(StatusCode::BAD_REQUEST, "request body framing is invalid\n");
            }
            Err(error) if error.is_connection_saturated() => {
                context
                    .metrics
                    .record_upstream_rejection(UpstreamRejection::ConnectionCapacity);
                attempt.finish(UpstreamResult::RequestFailure);
                upstream.abandon_probe(selected);
                return upstream_unavailable_response(
                    "upstream connection capacity is saturated\n",
                );
            }
            Err(error) if error.is_timeout() => {
                attempt.finish(UpstreamResult::Timeout);
                upstream.record_failure(selected);
                if upstream.retry.timeout {
                    if let Some(next) = upstream.next_retry_target(
                        &mut attempted,
                        RetryTargetContext {
                            client_ip: context.client_ip,
                            attempts_started,
                            maximum_attempts,
                            deadline: retry_deadline,
                            metrics: context.metrics,
                            reason: UpstreamRetryReason::Timeout,
                        },
                    ) {
                        selected = next;
                        target_activity = upstream.claim_target_activity(selected.index);
                        target_url = match build_retry_target_url(context, selected, &request_parts)
                        {
                            Ok(url) => url,
                            Err(()) => {
                                upstream.abandon_probe(selected);
                                return text_response(
                                    StatusCode::BAD_GATEWAY,
                                    "invalid upstream target\n",
                                );
                            }
                        };
                        continue;
                    }
                }
                return text_response(StatusCode::GATEWAY_TIMEOUT, "upstream timed out\n");
            }
            Err(_) => {
                attempt.finish(UpstreamResult::TransportFailure);
                upstream.record_failure(selected);
                if upstream.retry.transport_failure {
                    if let Some(next) = upstream.next_retry_target(
                        &mut attempted,
                        RetryTargetContext {
                            client_ip: context.client_ip,
                            attempts_started,
                            maximum_attempts,
                            deadline: retry_deadline,
                            metrics: context.metrics,
                            reason: UpstreamRetryReason::TransportFailure,
                        },
                    ) {
                        selected = next;
                        target_activity = upstream.claim_target_activity(selected.index);
                        target_url = match build_retry_target_url(context, selected, &request_parts)
                        {
                            Ok(url) => url,
                            Err(()) => {
                                upstream.abandon_probe(selected);
                                return text_response(
                                    StatusCode::BAD_GATEWAY,
                                    "invalid upstream target\n",
                                );
                            }
                        };
                        continue;
                    }
                }
                return text_response(StatusCode::BAD_GATEWAY, "upstream failed\n");
            }
        };

        let upstream_responded_early = request_control.pause_if_incomplete();
        let (mut response_parts, response_body) = response.into_parts();
        let (response_headers, forbidden_response_trailers, declared_response_trailers) =
            match forwarded_response_headers(
                &response_parts.headers,
                maximum_trailer_bytes,
                maximum_trailers,
            ) {
                Ok(result) => result,
                Err(()) => {
                    attempt.finish(UpstreamResult::InvalidResponse);
                    upstream.record_failure(selected);
                    request_control.cancel_if_incomplete();
                    return text_response(
                        StatusCode::BAD_GATEWAY,
                        "upstream Trailer declaration is invalid\n",
                    );
                }
            };
        response_parts.headers = response_headers;
        if upstream.status_is_failure(response_parts.status) {
            upstream.record_failure(selected);
        } else {
            upstream.record_success(selected);
        }
        attempt.finish(UpstreamResult::Response);

        if retryable_request {
            if let Some(reason) = upstream.retry.status_reason(response_parts.status) {
                if let Some(next) = upstream.next_retry_target(
                    &mut attempted,
                    RetryTargetContext {
                        client_ip: context.client_ip,
                        attempts_started,
                        maximum_attempts,
                        deadline: retry_deadline,
                        metrics: context.metrics,
                        reason,
                    },
                ) {
                    drop(response_body);
                    selected = next;
                    target_activity = upstream.claim_target_activity(selected.index);
                    target_url = match build_retry_target_url(context, selected, &request_parts) {
                        Ok(url) => url,
                        Err(()) => {
                            upstream.abandon_probe(selected);
                            return text_response(
                                StatusCode::BAD_GATEWAY,
                                "invalid upstream target\n",
                            );
                        }
                    };
                    continue;
                }
            }
        }

        if upstream_responded_early && request_version != Version::HTTP_2 {
            response_parts
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
                response_body,
                response_trailer_policy,
                request_control,
            )
        } else {
            GuardedProxyBody::response(response_body, response_trailer_policy)
        };
        return hold_upstream_permit(
            Response::from_parts(response_parts, Body::new(guarded_body)),
            upstream_permit,
            (context.generation.clone(), target_activity),
        );
    }
}

fn is_bodyless_idempotent_request(request: &Request<Body>) -> bool {
    request.body().is_end_stream()
        && matches!(
            *request.method(),
            Method::GET
                | Method::HEAD
                | Method::OPTIONS
                | Method::TRACE
                | Method::PUT
                | Method::DELETE
        )
}

fn build_retry_target_url(
    context: &ProxyRequestContext<'_>,
    selected: SelectedTarget<'_>,
    request_parts: &axum::http::request::Parts,
) -> Result<Url, ()> {
    build_target_url(
        selected.url,
        context.strip_prefix,
        &context.route.route_match.path,
        request_parts.uri.path(),
        context.normalized_path,
        request_parts.uri.query(),
    )
}

async fn proxy_websocket_request(
    context: &ProxyRequestContext<'_>,
    upstream: &ProxyUpstream,
    selected: SelectedTarget<'_>,
    target_activity: TargetActivityLease,
    upstream_permit: OwnedSemaphorePermit,
    target_url: Url,
    mut request: Request<Body>,
) -> Response<Body> {
    let downstream_upgrade = hyper::upgrade::on(&mut request);
    let (parts, _body) = request.into_parts();
    let maximum_trailer_bytes = context.generation.app.config().limits.max_trailer_bytes;
    let maximum_trailers = context.generation.app.config().limits.max_trailers;
    let (mut headers, _, _) = match forwarded_request_headers(
        &parts.headers,
        context.client_ip,
        context.external_scheme,
        context.external_authority,
        maximum_trailer_bytes,
        maximum_trailers,
    ) {
        Ok(headers) => headers,
        Err(()) => {
            upstream.abandon_probe(selected);
            return websocket_failure_response(text_response(
                StatusCode::BAD_REQUEST,
                "invalid WebSocket handshake\n",
            ));
        }
    };
    headers.remove(TE);
    headers.insert(CONNECTION, HeaderValue::from_static("upgrade"));
    headers.insert(UPGRADE, HeaderValue::from_static("websocket"));

    let target_uri = match target_url.as_str().parse() {
        Ok(uri) => uri,
        Err(_) => {
            upstream.abandon_probe(selected);
            return websocket_failure_response(text_response(
                StatusCode::BAD_GATEWAY,
                "invalid upstream target\n",
            ));
        }
    };
    let mut upstream_request = Request::new(Body::empty());
    *upstream_request.method_mut() = Method::GET;
    *upstream_request.version_mut() = Version::HTTP_11;
    *upstream_request.uri_mut() = target_uri;
    *upstream_request.headers_mut() = headers;

    let attempt = context.metrics.begin_upstream_attempt();
    let mut response = match upstream.client.execute(upstream_request).await {
        Ok(response) => response,
        Err(error) if error.is_connection_saturated() => {
            context
                .metrics
                .record_upstream_rejection(UpstreamRejection::ConnectionCapacity);
            attempt.finish(UpstreamResult::RequestFailure);
            upstream.abandon_probe(selected);
            return websocket_failure_response(upstream_unavailable_response(
                "upstream connection capacity is saturated\n",
            ));
        }
        Err(error) if error.is_timeout() => {
            attempt.finish(UpstreamResult::Timeout);
            upstream.record_failure(selected);
            return websocket_failure_response(text_response(
                StatusCode::GATEWAY_TIMEOUT,
                "upstream timed out\n",
            ));
        }
        Err(_) => {
            attempt.finish(UpstreamResult::TransportFailure);
            upstream.record_failure(selected);
            return websocket_failure_response(text_response(
                StatusCode::BAD_GATEWAY,
                "upstream failed\n",
            ));
        }
    };

    if response.status() != StatusCode::SWITCHING_PROTOCOLS {
        return forward_websocket_rejection(
            context,
            upstream,
            selected,
            target_activity,
            upstream_permit,
            response,
            attempt,
        );
    }
    if !valid_websocket_upgrade_response(&response) {
        attempt.finish(UpstreamResult::InvalidResponse);
        upstream.record_failure(selected);
        return websocket_failure_response(text_response(
            StatusCode::BAD_GATEWAY,
            "upstream failed\n",
        ));
    }

    let upstream_upgrade = hyper::upgrade::on(&mut response);
    let (mut parts, _body) = response.into_parts();
    let (mut headers, _, _) =
        match forwarded_response_headers(&parts.headers, maximum_trailer_bytes, maximum_trailers) {
            Ok(headers) => headers,
            Err(()) => {
                attempt.finish(UpstreamResult::InvalidResponse);
                upstream.record_failure(selected);
                return websocket_failure_response(text_response(
                    StatusCode::BAD_GATEWAY,
                    "upstream failed\n",
                ));
            }
        };
    headers.insert(CONNECTION, HeaderValue::from_static("upgrade"));
    headers.insert(UPGRADE, HeaderValue::from_static("websocket"));
    parts.headers = headers;
    upstream.record_success(selected);
    attempt.finish(UpstreamResult::Response);

    if context
        .tunnel_supervisor
        .try_spawn(
            downstream_upgrade,
            upstream_upgrade,
            upstream_permit,
            context.generation.clone(),
            target_activity,
        )
        .is_err()
    {
        return websocket_failure_response(upstream_unavailable_response(
            "server is shutting down\n",
        ));
    }
    Response::from_parts(parts, Body::empty())
}

fn forward_websocket_rejection(
    context: &ProxyRequestContext<'_>,
    upstream: &ProxyUpstream,
    selected: SelectedTarget<'_>,
    target_activity: TargetActivityLease,
    upstream_permit: OwnedSemaphorePermit,
    response: Response<UpstreamResponseBody>,
    attempt: UpstreamMetricLease,
) -> Response<Body> {
    let maximum_trailer_bytes = context.generation.app.config().limits.max_trailer_bytes;
    let maximum_trailers = context.generation.app.config().limits.max_trailers;
    let (mut parts, body) = response.into_parts();
    let (headers, forbidden_trailers, declared_trailers) =
        match forwarded_response_headers(&parts.headers, maximum_trailer_bytes, maximum_trailers) {
            Ok(headers) => headers,
            Err(()) => {
                attempt.finish(UpstreamResult::InvalidResponse);
                upstream.record_failure(selected);
                return websocket_failure_response(text_response(
                    StatusCode::BAD_GATEWAY,
                    "upstream failed\n",
                ));
            }
        };
    parts.headers = headers;
    parts
        .headers
        .insert(CONNECTION, HeaderValue::from_static("close"));
    let body = GuardedProxyBody::response(
        body,
        ProxyTrailerPolicy::new(
            maximum_trailer_bytes,
            maximum_trailers,
            declared_trailers,
            forbidden_trailers,
        ),
    );
    if upstream.status_is_failure(parts.status) {
        upstream.record_failure(selected);
    } else {
        upstream.record_success(selected);
    }
    attempt.finish(UpstreamResult::Response);
    hold_upstream_permit(
        Response::from_parts(parts, Body::new(body)),
        upstream_permit,
        (context.generation.clone(), target_activity),
    )
}

fn upgrade_failure_response(
    upgrade: UpgradeDisposition,
    response: Response<Body>,
) -> Response<Body> {
    if upgrade == UpgradeDisposition::WebSocket {
        websocket_failure_response(response)
    } else {
        response
    }
}

fn websocket_failure_response(mut response: Response<Body>) -> Response<Body> {
    response
        .headers_mut()
        .insert(CONNECTION, HeaderValue::from_static("close"));
    response
}

fn classify_upgrade_request(request: &Request<Body>) -> Result<UpgradeDisposition, ()> {
    let upgrade = single_upgrade_protocol(request.headers())?;
    let connection_upgrade = connection_contains_upgrade(request.headers())?;
    let Some(upgrade) = upgrade else {
        return if connection_upgrade {
            Err(())
        } else {
            Ok(UpgradeDisposition::None)
        };
    };
    if !connection_upgrade
        || request.version() != Version::HTTP_11
        || request.method() != Method::GET
        || request.headers().contains_key(CONTENT_LENGTH)
        || request.headers().contains_key(TRANSFER_ENCODING)
        || request.headers().contains_key(EXPECT)
    {
        return Err(());
    }
    if upgrade.as_str().eq_ignore_ascii_case("websocket") {
        Ok(UpgradeDisposition::WebSocket)
    } else {
        Ok(UpgradeDisposition::Unsupported)
    }
}

fn single_upgrade_protocol(headers: &HeaderMap) -> Result<Option<HeaderName>, ()> {
    let mut values = headers.get_all(UPGRADE).iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() {
        return Err(());
    }
    let value = trim_ascii_whitespace(value.as_bytes());
    if value.is_empty() || value.contains(&b',') {
        return Err(());
    }
    HeaderName::from_bytes(value).map(Some).map_err(|_| ())
}

fn connection_contains_upgrade(headers: &HeaderMap) -> Result<bool, ()> {
    let mut found = false;
    for value in headers.get_all(CONNECTION) {
        let value = value.to_str().map_err(|_| ())?;
        for token in value.split(',') {
            let token = token.trim();
            if token.is_empty() || HeaderName::from_bytes(token.as_bytes()).is_err() {
                return Err(());
            }
            found |= token.eq_ignore_ascii_case("upgrade");
        }
    }
    Ok(found)
}

fn valid_websocket_upgrade_response(response: &Response<UpstreamResponseBody>) -> bool {
    response.version() == Version::HTTP_11
        && matches!(connection_contains_upgrade(response.headers()), Ok(true))
        && matches!(single_upgrade_protocol(response.headers()), Ok(Some(protocol)) if protocol.as_str().eq_ignore_ascii_case("websocket"))
        && !response.headers().contains_key(CONTENT_LENGTH)
        && !response.headers().contains_key(TRANSFER_ENCODING)
}

fn trim_ascii_whitespace(mut value: &[u8]) -> &[u8] {
    while value.first().is_some_and(u8::is_ascii_whitespace) {
        value = &value[1..];
    }
    while value.last().is_some_and(u8::is_ascii_whitespace) {
        value = &value[..value.len() - 1];
    }
    value
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
    client_ip: IpAddr,
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
    if let Ok(value) = HeaderValue::from_str(&client_ip.to_string()) {
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

fn upstream_unavailable_response(body: &'static str) -> Response<Body> {
    let mut response = text_response(StatusCode::SERVICE_UNAVAILABLE, body);
    response
        .headers_mut()
        .insert(RETRY_AFTER, HeaderValue::from_static("1"));
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
mod tests;
