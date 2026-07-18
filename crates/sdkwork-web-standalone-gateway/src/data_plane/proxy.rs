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

impl ProxyUpstream<UpstreamClient> {
    pub(crate) fn build(
        app: &CompiledWebServerApp,
        config: &UpstreamConfig,
        resolver: Arc<BoundedSystemResolver>,
        metrics: Arc<DataPlaneMetrics>,
    ) -> Result<Self, DataPlaneError> {
        let resolver = Arc::new(GuardedDnsResolver::new_observed(
            resolver,
            config.address_policy.clone(),
            metrics,
        ));
        let client = UpstreamClient::build(app, config, resolver)?;
        let targets = config
            .targets
            .iter()
            .map(|target| {
                Url::parse(&target.url)
                    .map(|url| ProxyTarget {
                        active_health_url: config
                            .active_health
                            .as_ref()
                            .map(|policy| active_health_url(&url, &policy.uri)),
                        url,
                        weight: usize::from(target.weight),
                        backup: target.backup,
                        slow_start_duration_ms: target.slow_start_ms.unwrap_or(0),
                        slow_start_started_ms: AtomicU64::new(0),
                        active_requests: Arc::new(AtomicUsize::new(0)),
                        consecutive_failures: AtomicU32::new(0),
                        ejected_until_ms: AtomicU64::new(0),
                        probe_in_flight: AtomicBool::new(false),
                        active_available: AtomicBool::new(true),
                        active_failures: AtomicU32::new(0),
                        active_successes: AtomicU32::new(0),
                    })
                    .map_err(|_| DataPlaneError::InvalidUpstreamTarget {
                        upstream_id: config.id.clone(),
                        target: target.url.clone(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            id: config.id.clone(),
            client,
            smooth_weighted: SmoothWeightedState::new(targets.len()),
            targets,
            load_balancing: config.load_balancing,
            cursor: AtomicUsize::new(0),
            random_state: AtomicU64::new(scheduling_seed()),
            permits: Arc::new(Semaphore::new(config.max_in_flight_requests)),
            max_in_flight_requests: config.max_in_flight_requests,
            retry: RetryPolicy::from_config(config),
            health: PassiveHealthPolicy {
                failure_threshold: config.passive_health.failure_threshold,
                ejection_time_ms: config.passive_health.ejection_time_ms,
                failure_statuses: config.passive_health.failure_statuses.clone(),
            },
            active_health: config.active_health.as_ref().map(ActiveHealthPolicy::from),
            epoch: Instant::now(),
        })
    }

    pub(super) fn connection_capacity(&self) -> [u64; 3] {
        self.client.connection_capacity()
    }

    pub(super) fn target_connection_capacity(&self) -> [u64; 3] {
        self.client.target_connection_capacity()
    }

    pub(super) async fn run_active_health_check(
        &self,
        target_index: usize,
    ) -> ActiveHealthTransition {
        let Some(policy) = &self.active_health else {
            return ActiveHealthTransition::Unchanged;
        };
        let Some(target) = self.targets.get(target_index) else {
            return ActiveHealthTransition::Unchanged;
        };
        let Some(url) = target.active_health_url.clone() else {
            return ActiveHealthTransition::Unchanged;
        };
        let method = match policy.method {
            UpstreamActiveHealthMethod::Get => axum::http::Method::GET,
            UpstreamActiveHealthMethod::Head => axum::http::Method::HEAD,
        };
        let request = match Request::builder()
            .method(method)
            .uri(url.as_str())
            .body(Body::empty())
        {
            Ok(request) => request,
            Err(_) => return self.record_active_health(target_index, false),
        };
        let result = self
            .client
            .execute_with_timeout(request, policy.timeout)
            .await;
        let success = match result {
            Err(error) if error.is_connection_saturated() => {
                return ActiveHealthTransition::Unchanged
            }
            Ok(response)
                if response.status().as_u16() >= policy.success_status_min
                    && response.status().as_u16() <= policy.success_status_max =>
            {
                response_body_within_limit(response, policy.max_response_body_bytes).await
            }
            _ => false,
        };
        self.record_active_health(target_index, success)
    }
}

impl<T> ProxyUpstream<T> {
    pub(super) fn id(&self) -> &str {
        &self.id
    }

    pub(super) fn target_count(&self) -> usize {
        self.targets.len()
    }

    pub(super) fn active_health_interval(&self) -> Option<Duration> {
        self.active_health.as_ref().map(|policy| policy.interval)
    }

    pub(super) fn aggregate_target_health(&self) -> [u64; 4] {
        self.targets.iter().fold([0_u64; 4], |mut counts, target| {
            let state = if !target.active_available.load(Ordering::Acquire) {
                3
            } else if target.probe_in_flight.load(Ordering::Acquire) {
                2
            } else if target.ejected_until_ms.load(Ordering::Acquire) != 0 {
                1
            } else {
                0
            };
            counts[state] = counts[state].saturating_add(1);
            counts
        })
    }

    pub(super) fn request_capacity(&self) -> [u64; 3] {
        capacity_snapshot(
            self.max_in_flight_requests,
            self.permits.available_permits(),
        )
    }

    fn try_admit(&self) -> Result<OwnedSemaphorePermit, ()> {
        self.permits.clone().try_acquire_owned().map_err(|_| ())
    }

    #[cfg(test)]
    fn select_target(&self) -> Option<SelectedTarget<'_>> {
        self.select_target_observed(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), None)
    }

    fn select_target_observed(
        &self,
        client_ip: IpAddr,
        metrics: Option<&DataPlaneMetrics>,
    ) -> Option<SelectedTarget<'_>> {
        self.select_target_excluding_observed(&AttemptedTargets::default(), client_ip, metrics)
    }

    #[cfg(test)]
    fn select_target_excluding(&self, attempted: &AttemptedTargets) -> Option<SelectedTarget<'_>> {
        self.select_target_excluding_observed(
            attempted,
            IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            None,
        )
    }

    fn select_target_excluding_observed(
        &self,
        attempted: &AttemptedTargets,
        client_ip: IpAddr,
        metrics: Option<&DataPlaneMetrics>,
    ) -> Option<SelectedTarget<'_>> {
        let now_ms = self.now_ms();
        let use_backups = !self.targets.iter().enumerate().any(|(index, target)| {
            !attempted.contains(index) && !target.backup && target.is_eligible(now_ms)
        });
        match self.load_balancing {
            UpstreamLoadBalancingStrategy::RoundRobin => {
                self.select_smooth_weighted(attempted, now_ms, use_backups, metrics)
            }
            UpstreamLoadBalancingStrategy::LeastConnections => {
                self.select_weighted_least_connections(attempted, now_ms, use_backups)
            }
            UpstreamLoadBalancingStrategy::RandomTwoLeastConnections => {
                self.select_random_two_least_connections(attempted, now_ms, use_backups)
            }
            UpstreamLoadBalancingStrategy::IpHash => {
                self.select_ip_hash(attempted, now_ms, use_backups, client_ip, metrics)
            }
        }
    }

    fn select_smooth_weighted(
        &self,
        attempted: &AttemptedTargets,
        now_ms: u64,
        use_backups: bool,
        metrics: Option<&DataPlaneMetrics>,
    ) -> Option<SelectedTarget<'_>> {
        let selection = self
            .smooth_weighted
            .select(&self.targets, attempted, now_ms, use_backups);
        if selection.contended {
            if let Some(metrics) = metrics {
                metrics.record_upstream_selection_contention();
            }
        }
        selection.target
    }

    fn select_ip_hash(
        &self,
        attempted: &AttemptedTargets,
        now_ms: u64,
        use_backups: bool,
        client_ip: IpAddr,
        metrics: Option<&DataPlaneMetrics>,
    ) -> Option<SelectedTarget<'_>> {
        let total_weight = self
            .targets
            .iter()
            .filter(|target| target.backup == use_backups)
            .fold(0_usize, |total, target| total.saturating_add(target.weight));
        if total_weight == 0 {
            return None;
        }

        let mut hash = 89_usize;
        for _ in 0..=20 {
            hash = advance_ip_hash(hash, client_ip);
            let mut ticket = hash % total_weight;
            let mut selected_index = None;
            for (index, target) in self.targets.iter().enumerate() {
                if target.backup != use_backups {
                    continue;
                }
                if ticket < target.weight {
                    selected_index = Some(index);
                    break;
                }
                ticket -= target.weight;
            }
            let Some(index) = selected_index else {
                break;
            };
            if attempted.contains(index) || !self.targets[index].is_eligible(now_ms) {
                continue;
            }
            if let Some(selected) = self.targets[index].try_select(index, now_ms) {
                return Some(selected);
            }
        }

        self.select_smooth_weighted(attempted, now_ms, use_backups, metrics)
    }

    fn select_random_two_least_connections(
        &self,
        attempted: &AttemptedTargets,
        now_ms: u64,
        use_backups: bool,
    ) -> Option<SelectedTarget<'_>> {
        let (eligible_count, total_weight) = self.targets.iter().enumerate().fold(
            (0_usize, 0_usize),
            |(count, total), (index, target)| {
                if attempted.contains(index)
                    || target.backup != use_backups
                    || !target.is_eligible(now_ms)
                {
                    (count, total)
                } else {
                    (
                        count.saturating_add(1),
                        total.saturating_add(target.effective_weight(now_ms)),
                    )
                }
            },
        );
        if eligible_count == 0 || total_weight == 0 {
            return None;
        }

        let first =
            self.weighted_random_candidate(attempted, now_ms, use_backups, None, total_weight);
        let Some((first_index, first_weight)) = first else {
            return self.select_weighted_least_connections(attempted, now_ms, use_backups);
        };
        if eligible_count == 1 {
            return self.targets[first_index]
                .try_select(first_index, now_ms)
                .or_else(|| {
                    self.select_weighted_least_connections(attempted, now_ms, use_backups)
                });
        }

        let remaining_weight = total_weight.saturating_sub(first_weight);
        if remaining_weight == 0 {
            return self.select_weighted_least_connections(attempted, now_ms, use_backups);
        }
        let Some((second_index, second_weight)) = self.weighted_random_candidate(
            attempted,
            now_ms,
            use_backups,
            Some(first_index),
            remaining_weight,
        ) else {
            return self.select_weighted_least_connections(attempted, now_ms, use_backups);
        };

        let first_active = self.targets[first_index]
            .active_requests
            .load(Ordering::Acquire);
        let second_active = self.targets[second_index]
            .active_requests
            .load(Ordering::Acquire);
        let (winner, alternative) =
            if weighted_load_cmp(first_active, first_weight, second_active, second_weight)
                == std::cmp::Ordering::Greater
            {
                (second_index, first_index)
            } else {
                (first_index, second_index)
            };
        self.targets[winner]
            .try_select(winner, now_ms)
            .or_else(|| self.targets[alternative].try_select(alternative, now_ms))
            .or_else(|| self.select_weighted_least_connections(attempted, now_ms, use_backups))
    }

    fn weighted_random_candidate(
        &self,
        attempted: &AttemptedTargets,
        now_ms: u64,
        use_backups: bool,
        excluded_index: Option<usize>,
        total_weight: usize,
    ) -> Option<(usize, usize)> {
        let mut ticket = self.random_below(total_weight);
        for (index, target) in self.targets.iter().enumerate() {
            if excluded_index == Some(index)
                || attempted.contains(index)
                || target.backup != use_backups
                || !target.is_eligible(now_ms)
            {
                continue;
            }
            let weight = target.effective_weight(now_ms);
            if ticket < weight {
                return Some((index, weight));
            }
            ticket -= weight;
        }
        None
    }

    fn random_below(&self, upper_bound: usize) -> usize {
        debug_assert!(upper_bound != 0);
        let value = splitmix64(
            self.random_state
                .fetch_add(SPLITMIX64_GAMMA, Ordering::Relaxed),
        );
        ((u128::from(value) * upper_bound as u128) >> 64) as usize
    }

    fn select_weighted_least_connections(
        &self,
        attempted: &AttemptedTargets,
        now_ms: u64,
        use_backups: bool,
    ) -> Option<SelectedTarget<'_>> {
        let mut minimum: Option<(usize, usize)> = None;
        let mut tied_weight = 0usize;
        for (index, target) in self.targets.iter().enumerate() {
            if attempted.contains(index)
                || target.backup != use_backups
                || !target.is_eligible(now_ms)
            {
                continue;
            }
            let active = target.active_requests.load(Ordering::Acquire);
            let effective_weight = target.effective_weight(now_ms);
            match minimum {
                None => {
                    minimum = Some((active, effective_weight));
                    tied_weight = effective_weight;
                }
                Some((minimum_active, minimum_weight)) => {
                    match weighted_load_cmp(
                        active,
                        effective_weight,
                        minimum_active,
                        minimum_weight,
                    ) {
                        std::cmp::Ordering::Less => {
                            minimum = Some((active, effective_weight));
                            tied_weight = effective_weight;
                        }
                        std::cmp::Ordering::Equal => {
                            tied_weight = tied_weight.saturating_add(effective_weight);
                        }
                        std::cmp::Ordering::Greater => {}
                    }
                }
            }
        }
        let (minimum_active, minimum_weight) = minimum?;
        let ticket = self.cursor.fetch_add(1, Ordering::Relaxed) % tied_weight;
        let mut remaining = ticket;
        for (index, target) in self.targets.iter().enumerate() {
            if attempted.contains(index)
                || target.backup != use_backups
                || !target.is_eligible(now_ms)
            {
                continue;
            }
            let active = target.active_requests.load(Ordering::Acquire);
            let effective_weight = target.effective_weight(now_ms);
            if weighted_load_cmp(active, effective_weight, minimum_active, minimum_weight)
                != std::cmp::Ordering::Equal
            {
                continue;
            }
            if remaining < effective_weight {
                if let Some(selected) = target.try_select(index, now_ms) {
                    return Some(selected);
                }
                break;
            }
            remaining -= effective_weight;
        }

        self.targets
            .iter()
            .enumerate()
            .filter(|(index, target)| {
                !attempted.contains(*index)
                    && target.backup == use_backups
                    && target.is_eligible(now_ms)
            })
            .min_by(|(_, left), (_, right)| {
                weighted_load_cmp(
                    left.active_requests.load(Ordering::Acquire),
                    left.effective_weight(now_ms),
                    right.active_requests.load(Ordering::Acquire),
                    right.effective_weight(now_ms),
                )
            })
            .and_then(|(index, target)| target.try_select(index, now_ms))
    }

    fn claim_target_activity(&self, target_index: usize) -> TargetActivityLease {
        TargetActivityLease::claim(&self.targets[target_index].active_requests)
    }

    fn next_retry_target(
        &self,
        attempted: &mut AttemptedTargets,
        context: RetryTargetContext<'_>,
    ) -> Option<SelectedTarget<'_>> {
        if context.attempts_started >= context.maximum_attempts
            || Instant::now() >= context.deadline
        {
            return None;
        }
        let selected = self.select_target_excluding_observed(
            attempted,
            context.client_ip,
            Some(context.metrics),
        )?;
        attempted.insert(selected.index);
        context.metrics.record_upstream_retry(context.reason);
        Some(selected)
    }

    fn status_is_failure(&self, status: StatusCode) -> bool {
        self.health.failure_statuses.contains(&status.as_u16())
    }

    fn record_success(&self, selected: SelectedTarget<'_>) {
        let target = &self.targets[selected.index];
        let recovered = if selected.ejection_deadline_ms == 0 {
            false
        } else {
            let mut smooth = self.smooth_weighted.lock_target(selected.index);
            let recovered = target
                .ejected_until_ms
                .compare_exchange(
                    selected.ejection_deadline_ms,
                    0,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok();
            if recovered {
                target.consecutive_failures.store(0, Ordering::Release);
                if target.active_available.load(Ordering::Acquire) {
                    target.start_slow_start(self.now_ms());
                }
                smooth.reset(target.slow_start_marker());
            }
            recovered
        };
        if recovered
            || (selected.ejection_deadline_ms == 0
                && target.ejected_until_ms.load(Ordering::Acquire) == 0)
        {
            target.consecutive_failures.store(0, Ordering::Release);
        }
        if selected.probe {
            target.probe_in_flight.store(false, Ordering::Release);
        }
    }

    fn record_failure(&self, selected: SelectedTarget<'_>) {
        let target = &self.targets[selected.index];
        if target.ejected_until_ms.load(Ordering::Acquire) != selected.ejection_deadline_ms {
            if selected.probe {
                target.probe_in_flight.store(false, Ordering::Release);
            }
            return;
        }
        let failures = if selected.probe {
            self.health.failure_threshold
        } else {
            target
                .consecutive_failures
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                    Some(current.saturating_add(1))
                })
                .unwrap_or(u32::MAX)
                .saturating_add(1)
        };
        if failures >= self.health.failure_threshold {
            let next_deadline = self.now_ms().saturating_add(self.health.ejection_time_ms);
            let mut smooth = self.smooth_weighted.lock_target(selected.index);
            if target
                .ejected_until_ms
                .compare_exchange(
                    selected.ejection_deadline_ms,
                    next_deadline,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                target.consecutive_failures.store(0, Ordering::Release);
                target.slow_start_started_ms.store(0, Ordering::Release);
                smooth.reset(0);
            }
        }
        if selected.probe {
            target.probe_in_flight.store(false, Ordering::Release);
        }
    }

    fn abandon_probe(&self, selected: SelectedTarget<'_>) {
        if selected.probe {
            self.targets[selected.index]
                .probe_in_flight
                .store(false, Ordering::Release);
        }
    }

    fn probe_claim_lease(&self, target_index: usize, claimed: bool) -> ProbeClaimLease<'_> {
        ProbeClaimLease {
            flag: claimed.then(|| &self.targets[target_index].probe_in_flight),
        }
    }

    fn now_ms(&self) -> u64 {
        self.epoch.elapsed().as_millis().min(u64::MAX as u128) as u64
    }

    fn record_active_health(&self, target_index: usize, success: bool) -> ActiveHealthTransition {
        let Some(policy) = &self.active_health else {
            return ActiveHealthTransition::Unchanged;
        };
        let target = &self.targets[target_index];
        if success {
            target.active_failures.store(0, Ordering::Release);
            if target.active_available.load(Ordering::Acquire) {
                target.active_successes.store(0, Ordering::Release);
                return ActiveHealthTransition::Unchanged;
            }
            let successes = saturating_increment(&target.active_successes);
            if successes >= policy.healthy_threshold {
                let mut smooth = self.smooth_weighted.lock_target(target_index);
                if target
                    .active_available
                    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
                {
                    target.active_successes.store(0, Ordering::Release);
                    if target.ejected_until_ms.load(Ordering::Acquire) == 0 {
                        target.start_slow_start(self.now_ms());
                        smooth.reset(target.slow_start_marker());
                    } else {
                        smooth.reset(0);
                    }
                    return ActiveHealthTransition::BecameHealthy;
                }
            }
            return ActiveHealthTransition::Unchanged;
        }

        target.active_successes.store(0, Ordering::Release);
        if !target.active_available.load(Ordering::Acquire) {
            target.active_failures.store(0, Ordering::Release);
            return ActiveHealthTransition::Unchanged;
        }
        let failures = saturating_increment(&target.active_failures);
        if failures >= policy.unhealthy_threshold {
            let mut smooth = self.smooth_weighted.lock_target(target_index);
            if target
                .active_available
                .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                target.active_failures.store(0, Ordering::Release);
                target.slow_start_started_ms.store(0, Ordering::Release);
                smooth.reset(0);
                return ActiveHealthTransition::BecameUnhealthy;
            }
        }
        ActiveHealthTransition::Unchanged
    }
}

fn weighted_load_cmp(
    left_active: usize,
    left_weight: usize,
    right_active: usize,
    right_weight: usize,
) -> std::cmp::Ordering {
    (left_active as u128 * right_weight as u128).cmp(&(right_active as u128 * left_weight as u128))
}

fn advance_ip_hash(hash: usize, client_ip: IpAddr) -> usize {
    match client_ip {
        IpAddr::V4(address) => address.octets()[..3]
            .iter()
            .fold(hash, |hash, byte| (hash * 113 + usize::from(*byte)) % 6271),
        IpAddr::V6(address) => address
            .octets()
            .iter()
            .fold(hash, |hash, byte| (hash * 113 + usize::from(*byte)) % 6271),
    }
}

fn scheduling_seed() -> u64 {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0);
    time ^ u64::from(std::process::id()).rotate_left(32)
        ^ RANDOM_SEED_SEQUENCE.fetch_add(SPLITMIX64_GAMMA, Ordering::Relaxed)
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(SPLITMIX64_GAMMA);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

impl ProxyTarget {
    fn start_slow_start(&self, now_ms: u64) {
        if self.slow_start_duration_ms != 0 {
            self.slow_start_started_ms
                .store(now_ms.max(1), Ordering::Release);
        }
    }

    pub(super) fn effective_weight(&self, now_ms: u64) -> usize {
        if self.slow_start_duration_ms == 0 {
            return self.weight;
        }
        let started_ms = self.slow_start_started_ms.load(Ordering::Acquire);
        if started_ms == 0 {
            return self.weight;
        }
        let elapsed_ms = now_ms.saturating_sub(started_ms);
        if elapsed_ms >= self.slow_start_duration_ms {
            return match self.slow_start_started_ms.compare_exchange(
                started_ms,
                0,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => self.weight,
                Err(current_started_ms) => self.ramped_weight(now_ms, current_started_ms),
            };
        }
        self.ramped_weight(now_ms, started_ms)
    }

    fn ramped_weight(&self, now_ms: u64, started_ms: u64) -> usize {
        if started_ms == 0 {
            return self.weight;
        }
        let elapsed_ms = now_ms.saturating_sub(started_ms);
        if elapsed_ms >= self.slow_start_duration_ms {
            return self.weight;
        }
        let scaled = (elapsed_ms as u128 * self.weight as u128
            / self.slow_start_duration_ms as u128) as usize;
        scaled.clamp(1, self.weight)
    }

    pub(super) fn slow_start_marker(&self) -> u64 {
        self.slow_start_started_ms.load(Ordering::Acquire)
    }

    pub(super) fn is_eligible(&self, now_ms: u64) -> bool {
        if !self.active_available.load(Ordering::Acquire) {
            return false;
        }
        let ejected_until = self.ejected_until_ms.load(Ordering::Acquire);
        ejected_until == 0
            || (ejected_until <= now_ms && !self.probe_in_flight.load(Ordering::Acquire))
    }

    pub(super) fn try_select(&self, index: usize, now_ms: u64) -> Option<SelectedTarget<'_>> {
        if !self.active_available.load(Ordering::Acquire) {
            return None;
        }
        let ejected_until = self.ejected_until_ms.load(Ordering::Acquire);
        if ejected_until == 0 {
            return Some(SelectedTarget {
                index,
                url: &self.url,
                probe: false,
                ejection_deadline_ms: 0,
            });
        }
        if ejected_until <= now_ms
            && self
                .probe_in_flight
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
        {
            return Some(SelectedTarget {
                index,
                url: &self.url,
                probe: true,
                ejection_deadline_ms: ejected_until,
            });
        }
        None
    }
}

impl From<&UpstreamActiveHealthConfig> for ActiveHealthPolicy {
    fn from(config: &UpstreamActiveHealthConfig) -> Self {
        Self {
            method: config.method,
            interval: Duration::from_millis(config.interval_ms),
            timeout: Duration::from_millis(config.timeout_ms),
            unhealthy_threshold: config.unhealthy_threshold,
            healthy_threshold: config.healthy_threshold,
            success_status_min: config.success_status_min,
            success_status_max: config.success_status_max,
            max_response_body_bytes: config.max_response_body_bytes,
        }
    }
}

fn active_health_url(target: &Url, uri: &str) -> Url {
    target
        .join(uri)
        .expect("semantic validation guarantees an origin-form active health URI")
}

async fn response_body_within_limit(
    response: Response<UpstreamResponseBody>,
    maximum_bytes: u64,
) -> bool {
    if response.headers().get(CONTENT_LENGTH).is_some_and(|value| {
        value
            .to_str()
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .is_some_and(|length| length > maximum_bytes)
    }) {
        return false;
    }
    let mut body = response.into_body();
    let mut observed = 0_u64;
    while let Some(frame) = body.frame().await {
        match frame {
            Ok(frame) => {
                observed =
                    observed.saturating_add(frame.data_ref().map_or(0, |data| data.len() as u64));
                if observed > maximum_bytes {
                    return false;
                }
            }
            Err(_) => return false,
        }
    }
    true
}

fn saturating_increment(counter: &AtomicU32) -> u32 {
    counter
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
            Some(current.saturating_add(1))
        })
        .unwrap_or(u32::MAX)
        .saturating_add(1)
}

fn capacity_snapshot(configured: usize, available: usize) -> [u64; 3] {
    let configured = configured as u64;
    let available = available.min(configured as usize) as u64;
    [configured, configured.saturating_sub(available), available]
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
mod tests {
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::{Duration, Instant},
    };

    use axum::{
        body::Body,
        http::{Method, Request, Version},
    };
    use tokio::sync::Semaphore;

    use super::{
        advance_ip_hash, build_target_url, classify_upgrade_request,
        is_bodyless_idempotent_request, ActiveHealthPolicy, ActiveHealthTransition, AtomicBool,
        AtomicU32, AtomicU64, AttemptedTargets, IpAddr, PassiveHealthPolicy, ProxyTarget,
        ProxyUpstream, RetryPolicy, SmoothWeightedState, TargetActivityLease, UpgradeDisposition,
        UpstreamActiveHealthMethod, UpstreamLoadBalancingStrategy, Url,
    };

    #[test]
    fn classic_websocket_upgrade_requires_http11_get_and_exact_tokens() {
        let valid = Request::builder()
            .method(Method::GET)
            .version(Version::HTTP_11)
            .header("connection", "keep-alive, Upgrade")
            .header("upgrade", "WebSocket")
            .body(Body::empty())
            .expect("valid request");
        assert_eq!(
            classify_upgrade_request(&valid),
            Ok(UpgradeDisposition::WebSocket)
        );

        for request in [
            Request::builder()
                .method(Method::GET)
                .version(Version::HTTP_2)
                .header("connection", "upgrade")
                .header("upgrade", "websocket")
                .body(Body::empty())
                .expect("HTTP/2 request"),
            Request::builder()
                .method(Method::POST)
                .version(Version::HTTP_11)
                .header("connection", "upgrade")
                .header("upgrade", "websocket")
                .body(Body::empty())
                .expect("POST request"),
            Request::builder()
                .method(Method::GET)
                .version(Version::HTTP_11)
                .header("connection", "x-upgrade")
                .header("upgrade", "websocket")
                .body(Body::empty())
                .expect("substring token request"),
            Request::builder()
                .method(Method::GET)
                .version(Version::HTTP_11)
                .header("connection", "upgrade")
                .header("upgrade", "websocket")
                .header("content-length", "0")
                .body(Body::empty())
                .expect("Body-framed request"),
        ] {
            assert_eq!(classify_upgrade_request(&request), Err(()));
        }
    }

    #[test]
    fn other_well_formed_upgrade_protocols_remain_explicitly_unsupported() {
        let request = Request::builder()
            .method(Method::GET)
            .version(Version::HTTP_11)
            .header("connection", "upgrade")
            .header("upgrade", "h2c")
            .body(Body::empty())
            .expect("unsupported upgrade request");
        assert_eq!(
            classify_upgrade_request(&request),
            Ok(UpgradeDisposition::Unsupported)
        );
    }

    fn test_upstream(
        target_count: usize,
        threshold: u32,
        ejection_time_ms: u64,
    ) -> ProxyUpstream<()> {
        test_weighted_upstream(&vec![1; target_count], threshold, ejection_time_ms)
    }

    fn test_weighted_upstream(
        weights: &[u16],
        threshold: u32,
        ejection_time_ms: u64,
    ) -> ProxyUpstream<()> {
        ProxyUpstream {
            id: "test-upstream".to_owned(),
            client: (),
            targets: weights
                .iter()
                .enumerate()
                .map(|(index, weight)| ProxyTarget {
                    url: Url::parse(&format!("http://127.0.0.1:{}/", 10_000 + index))
                        .expect("valid target URL"),
                    weight: usize::from(*weight),
                    backup: false,
                    slow_start_duration_ms: 0,
                    slow_start_started_ms: AtomicU64::new(0),
                    active_requests: Arc::new(AtomicUsize::new(0)),
                    consecutive_failures: AtomicU32::new(0),
                    ejected_until_ms: AtomicU64::new(0),
                    probe_in_flight: AtomicBool::new(false),
                    active_available: AtomicBool::new(true),
                    active_failures: AtomicU32::new(0),
                    active_successes: AtomicU32::new(0),
                    active_health_url: None,
                })
                .collect(),
            load_balancing: UpstreamLoadBalancingStrategy::RoundRobin,
            smooth_weighted: SmoothWeightedState::new(weights.len()),
            cursor: AtomicUsize::new(0),
            random_state: AtomicU64::new(0),
            permits: Arc::new(Semaphore::new(1)),
            max_in_flight_requests: 1,
            retry: RetryPolicy {
                enabled: false,
                maximum_attempts: 1,
                total_timeout: Duration::from_secs(1),
                attempt_timeout: Duration::from_secs(1),
                transport_failure: false,
                timeout: false,
                statuses: [false; 3],
            },
            health: PassiveHealthPolicy {
                failure_threshold: threshold,
                ejection_time_ms,
                failure_statuses: vec![503],
            },
            active_health: None,
            epoch: Instant::now(),
        }
    }

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

    #[tokio::test]
    async fn admission_never_queues_and_releases_on_drop() {
        let upstream = test_upstream(1, 1, 10);
        assert_eq!(upstream.request_capacity(), [1, 0, 1]);
        let permit = upstream.try_admit().expect("first request is admitted");
        assert_eq!(upstream.request_capacity(), [1, 1, 0]);
        assert!(upstream.try_admit().is_err());
        drop(permit);
        assert_eq!(upstream.request_capacity(), [1, 0, 1]);
        assert!(upstream.try_admit().is_ok());
    }

    #[tokio::test]
    async fn cancelled_attempt_releases_half_open_probe_claim() {
        let upstream = test_upstream(1, 1, 10);
        upstream.targets[0]
            .probe_in_flight
            .store(true, Ordering::Release);

        let pending_attempt = async {
            let _lease = upstream.probe_claim_lease(0, true);
            std::future::pending::<()>().await;
        };
        let mut pending_attempt = Box::pin(pending_attempt);
        assert!(
            tokio::time::timeout(Duration::from_millis(1), pending_attempt.as_mut())
                .await
                .is_err()
        );
        assert!(upstream.targets[0].probe_in_flight.load(Ordering::Acquire));

        drop(pending_attempt);
        assert!(!upstream.targets[0].probe_in_flight.load(Ordering::Acquire));
    }

    #[test]
    fn retry_selection_never_reuses_a_target_and_request_replay_is_fail_closed() {
        let upstream = test_upstream(3, 1, 10);
        let mut attempted = AttemptedTargets::default();
        let first = upstream.select_target().expect("first target");
        attempted.insert(first.index);
        let second = upstream
            .select_target_excluding(&attempted)
            .expect("different second target");
        assert_ne!(first.index, second.index);
        attempted.insert(second.index);
        let third = upstream
            .select_target_excluding(&attempted)
            .expect("different third target");
        assert_ne!(first.index, third.index);
        assert_ne!(second.index, third.index);

        let empty_get = Request::builder()
            .method(Method::GET)
            .body(Body::empty())
            .expect("empty GET");
        assert!(is_bodyless_idempotent_request(&empty_get));
        let post = Request::builder()
            .method(Method::POST)
            .body(Body::empty())
            .expect("POST");
        assert!(!is_bodyless_idempotent_request(&post));
        let body_get = Request::builder()
            .method(Method::GET)
            .body(Body::from("payload"))
            .expect("body GET");
        assert!(!is_bodyless_idempotent_request(&body_get));
    }

    #[tokio::test]
    async fn backup_tier_waits_for_primary_exhaustion_and_yields_to_half_open_probe() {
        let mut upstream = test_upstream(3, 1, 10);
        upstream.targets[2].backup = true;
        let mut attempted = AttemptedTargets::default();

        let first = upstream.select_target().expect("first primary target");
        assert_eq!(first.index, 0);
        attempted.insert(first.index);
        let second = upstream
            .select_target_excluding(&attempted)
            .expect("second primary precedes backup");
        assert_eq!(second.index, 1);
        attempted.insert(second.index);
        let backup = upstream
            .select_target_excluding(&attempted)
            .expect("backup follows exhausted primaries");
        assert_eq!(backup.index, 2);

        let mut health_upstream = test_upstream(2, 1, 10);
        health_upstream.targets[1].backup = true;
        let primary = health_upstream.select_target().expect("healthy primary");
        assert_eq!(primary.index, 0);
        health_upstream.record_failure(primary);
        let backup = health_upstream
            .select_target()
            .expect("backup serves during primary ejection");
        assert_eq!(backup.index, 1);
        health_upstream.record_success(backup);

        tokio::time::sleep(Duration::from_millis(15)).await;
        let probe = health_upstream
            .select_target()
            .expect("expired primary probe precedes backup");
        assert_eq!(probe.index, 0);
        assert!(probe.probe);
        health_upstream.record_success(probe);
        assert_eq!(
            health_upstream
                .select_target()
                .expect("recovered primary resumes traffic")
                .index,
            0
        );
    }

    #[test]
    fn weighted_selection_honors_relative_slots_and_equal_weight_round_robin() {
        let weighted = test_weighted_upstream(&[3, 1], 3, 1_000);
        let selected = (0..8)
            .map(|_| weighted.select_target().expect("weighted target").index)
            .collect::<Vec<_>>();
        assert_eq!(selected, [0, 0, 1, 0, 0, 0, 1, 0]);

        let equal = test_weighted_upstream(&[1, 1, 1], 3, 1_000);
        let selected = (0..6)
            .map(|_| equal.select_target().expect("equal target").index)
            .collect::<Vec<_>>();
        assert_eq!(selected, [0, 1, 2, 0, 1, 2]);

        let three_way = test_weighted_upstream(&[5, 1, 1], 3, 1_000);
        let selected = (0..7)
            .map(|_| three_way.select_target().expect("smooth target").index)
            .collect::<Vec<_>>();
        assert_eq!(selected, [0, 0, 1, 0, 2, 0, 0]);
    }

    #[test]
    fn attempted_exclusion_retains_the_global_smooth_phase() {
        let upstream = test_weighted_upstream(&[3, 1], 3, 1_000);
        assert_eq!(upstream.select_target().expect("first target").index, 0);
        assert_eq!(upstream.select_target().expect("second target").index, 0);

        let mut attempted = AttemptedTargets::default();
        attempted.insert(0);
        assert_eq!(
            upstream
                .select_target_excluding(&attempted)
                .expect("retry target")
                .index,
            1
        );
        assert_eq!(
            upstream
                .select_target()
                .expect("retained phase target")
                .index,
            1,
            "request-local exclusion must not reset the skipped target's current weight"
        );
        assert_eq!(
            upstream
                .select_target()
                .expect("completed smooth cycle target")
                .index,
            0
        );
    }

    #[test]
    fn primary_and_backup_tiers_retain_independent_smooth_phases() {
        let mut upstream = test_weighted_upstream(&[3, 1, 2, 1], 3, 1_000);
        upstream.targets[2].backup = true;
        upstream.targets[3].backup = true;

        assert_eq!(
            upstream
                .select_target()
                .expect("initial primary target")
                .index,
            0
        );
        let mut primaries_attempted = AttemptedTargets::default();
        primaries_attempted.insert(0);
        primaries_attempted.insert(1);
        assert_eq!(
            upstream
                .select_target_excluding(&primaries_attempted)
                .expect("initial backup target")
                .index,
            2
        );

        assert_eq!(
            upstream
                .select_target()
                .expect("retained primary phase")
                .index,
            0
        );
        assert_eq!(
            upstream
                .select_target()
                .expect("next retained primary phase")
                .index,
            1
        );
        assert_eq!(
            upstream
                .select_target_excluding(&primaries_attempted)
                .expect("retained backup phase")
                .index,
            3,
            "primary selections must not advance or reset the backup tier"
        );
    }

    #[tokio::test]
    async fn passive_recovery_resets_the_target_smooth_phase() {
        let upstream = test_weighted_upstream(&[3, 1], 1, 20);
        let failed = upstream.select_target().expect("initial weighted target");
        assert_eq!(failed.index, 0);
        upstream.record_failure(failed);

        let survivor = upstream
            .select_target()
            .expect("healthy target during ejection");
        assert_eq!(survivor.index, 1);
        upstream.record_success(survivor);

        tokio::time::sleep(Duration::from_millis(25)).await;
        let probe = upstream.select_target().expect("half-open recovery probe");
        assert_eq!(probe.index, 0);
        assert!(probe.probe);
        upstream.record_success(probe);

        assert_eq!(
            upstream
                .select_target()
                .expect("reset recovery phase target")
                .index,
            0
        );
        assert_eq!(
            upstream
                .select_target()
                .expect("smooth post-recovery target")
                .index,
            1
        );
    }

    #[test]
    fn active_recovery_resets_phase_and_uses_slow_start_weight() {
        let mut upstream = test_weighted_upstream(&[4, 1], 1, 1_000);
        upstream.targets[0].slow_start_duration_ms = 1_000;
        upstream.active_health = Some(ActiveHealthPolicy {
            method: UpstreamActiveHealthMethod::Get,
            interval: Duration::from_secs(1),
            timeout: Duration::from_millis(100),
            unhealthy_threshold: 1,
            healthy_threshold: 1,
            success_status_min: 200,
            success_status_max: 399,
            max_response_body_bytes: 64,
        });

        assert_eq!(upstream.select_target().expect("nominal target").index, 0);
        assert_eq!(
            upstream.record_active_health(0, false),
            ActiveHealthTransition::BecameUnhealthy
        );
        assert_eq!(
            upstream
                .select_target()
                .expect("survivor while target is unavailable")
                .index,
            1
        );
        assert_eq!(
            upstream.record_active_health(0, true),
            ActiveHealthTransition::BecameHealthy
        );
        assert_eq!(upstream.targets[0].effective_weight(upstream.now_ms()), 1);
        assert_eq!(
            upstream
                .select_target()
                .expect("first slow-start phase target")
                .index,
            1,
            "the recovered target must re-enter with effective weight one"
        );
        assert_eq!(
            upstream
                .select_target()
                .expect("next slow-start phase target")
                .index,
            0
        );
    }

    #[test]
    fn concurrent_smooth_selection_preserves_exact_weighted_totals_without_deadlock() {
        let upstream = Arc::new(test_weighted_upstream(&[3, 1], 3, 1_000));
        let counts = Arc::new([AtomicUsize::new(0), AtomicUsize::new(0)]);
        let start = Arc::new(std::sync::Barrier::new(8));
        let threads = (0..8)
            .map(|_| {
                let upstream = upstream.clone();
                let counts = counts.clone();
                let start = start.clone();
                std::thread::spawn(move || {
                    start.wait();
                    for _ in 0..10_000 {
                        let selected = upstream.select_target().expect("concurrent target");
                        counts[selected.index].fetch_add(1, Ordering::Relaxed);
                    }
                })
            })
            .collect::<Vec<_>>();
        for thread in threads {
            thread.join().expect("smooth selector thread joins");
        }
        assert_eq!(counts[0].load(Ordering::Acquire), 60_000);
        assert_eq!(counts[1].load(Ordering::Acquire), 20_000);
    }

    #[test]
    fn weighted_least_connections_uses_active_request_ratios_and_tier_health() {
        let mut upstream = test_weighted_upstream(&[2, 1, 100], 1, 1_000);
        upstream.load_balancing = UpstreamLoadBalancingStrategy::LeastConnections;
        upstream.targets[2].backup = true;

        upstream.targets[0]
            .active_requests
            .store(1, Ordering::Release);
        upstream.targets[1]
            .active_requests
            .store(1, Ordering::Release);
        let selected = upstream
            .select_target()
            .expect("weighted lower ratio target is selected");
        assert_eq!(selected.index, 0, "1/2 is less loaded than 1/1");

        upstream.targets[0]
            .active_requests
            .store(2, Ordering::Release);
        upstream.targets[1]
            .active_requests
            .store(1, Ordering::Release);
        upstream.cursor.store(0, Ordering::Release);
        let tied = (0..6)
            .map(|_| {
                upstream
                    .select_target()
                    .expect("equal weighted load remains selectable")
                    .index
            })
            .collect::<Vec<_>>();
        assert_eq!(tied, [0, 0, 1, 0, 0, 1]);
        assert!(tied.iter().all(|index| *index != 2));

        upstream.targets[0]
            .active_available
            .store(false, Ordering::Release);
        upstream.targets[1]
            .active_available
            .store(false, Ordering::Release);
        assert_eq!(
            upstream
                .select_target()
                .expect("backup becomes eligible after primary health exclusion")
                .index,
            2
        );
    }

    #[test]
    fn random_two_selects_the_lower_weighted_load_from_distinct_candidates() {
        let mut upstream = test_weighted_upstream(&[10, 1], 1, 1_000);
        upstream.load_balancing = UpstreamLoadBalancingStrategy::RandomTwoLeastConnections;
        upstream.targets[0]
            .active_requests
            .store(1, Ordering::Release);
        upstream.targets[1]
            .active_requests
            .store(1, Ordering::Release);
        for _ in 0..32 {
            assert_eq!(
                upstream
                    .select_target()
                    .expect("two weighted candidates")
                    .index,
                0,
                "1/10 is less loaded than 1/1 regardless of weighted sample order"
            );
        }

        upstream.targets[0]
            .active_requests
            .store(20, Ordering::Release);
        upstream.targets[1]
            .active_requests
            .store(0, Ordering::Release);
        for _ in 0..32 {
            assert_eq!(
                upstream
                    .select_target()
                    .expect("lower-load candidate")
                    .index,
                1
            );
        }
    }

    #[test]
    fn random_two_composes_attempted_primary_backup_and_single_target_rules() {
        let mut upstream = test_weighted_upstream(&[1, 1, 100], 1, 1_000);
        upstream.load_balancing = UpstreamLoadBalancingStrategy::RandomTwoLeastConnections;
        upstream.targets[2].backup = true;

        let mut attempted = AttemptedTargets::default();
        attempted.insert(0);
        assert_eq!(
            upstream
                .select_target_excluding(&attempted)
                .expect("single unattempted primary")
                .index,
            1
        );
        attempted.insert(1);
        assert_eq!(
            upstream
                .select_target_excluding(&attempted)
                .expect("backup after primary exhaustion")
                .index,
            2
        );
        attempted.insert(2);
        assert!(upstream.select_target_excluding(&attempted).is_none());
    }

    #[test]
    fn random_two_uses_slow_start_effective_weight_and_bounded_mapping() {
        let mut upstream = test_weighted_upstream(&[10, 1], 1, 1_000);
        upstream.load_balancing = UpstreamLoadBalancingStrategy::RandomTwoLeastConnections;
        upstream.targets[0].slow_start_duration_ms = 1_000;
        upstream.targets[0].start_slow_start(100);
        upstream.targets[0]
            .active_requests
            .store(2, Ordering::Release);
        upstream.targets[1]
            .active_requests
            .store(1, Ordering::Release);
        assert_eq!(
            upstream
                .select_random_two_least_connections(&AttemptedTargets::default(), 100, false,)
                .expect("slow-start candidates")
                .index,
            1,
            "during recovery 2/1 is more loaded than 1/1"
        );
        upstream.targets[0]
            .slow_start_started_ms
            .store(0, Ordering::Release);
        assert_eq!(
            upstream
                .select_target()
                .expect("nominal-weight candidates")
                .index,
            0,
            "at nominal weight 2/10 is less loaded than 1/1"
        );

        for bound in [1, 2, 3, 1_000, 1_000_000] {
            for _ in 0..1_000 {
                assert!(upstream.random_below(bound) < bound);
            }
        }
    }

    #[test]
    fn ip_hash_matches_nginx_ipv4_and_ipv6_vectors() {
        assert_eq!(
            advance_ip_hash(89, "192.0.2.1".parse::<IpAddr>().expect("IPv4")),
            6255
        );
        assert_eq!(
            advance_ip_hash(89, "192.0.2.254".parse::<IpAddr>().expect("IPv4")),
            6255,
            "the fourth IPv4 octet is intentionally excluded by Nginx ip_hash"
        );
        assert_eq!(
            advance_ip_hash(89, "2001:db8::1".parse::<IpAddr>().expect("IPv6")),
            2600
        );
        assert_eq!(
            advance_ip_hash(89, "2001:db8::2".parse::<IpAddr>().expect("IPv6")),
            2601,
            "all sixteen IPv6 bytes participate in the key"
        );
    }

    #[test]
    fn ip_hash_preserves_weighted_affinity_and_rehashes_unavailable_or_attempted_targets() {
        let mut upstream = test_weighted_upstream(&[3, 1], 1, 1_000);
        upstream.load_balancing = UpstreamLoadBalancingStrategy::IpHash;
        let affinity_ip = "10.20.2.1".parse::<IpAddr>().expect("affinity IPv4");
        assert_eq!(
            upstream
                .select_target_excluding_observed(&AttemptedTargets::default(), affinity_ip, None,)
                .expect("weighted affinity target")
                .index,
            1,
            "the first hash is 4827 and weighted ticket 3 maps to target one"
        );
        assert_eq!(
            upstream
                .select_target_excluding_observed(
                    &AttemptedTargets::default(),
                    "10.20.2.254".parse::<IpAddr>().expect("same /24 IPv4"),
                    None,
                )
                .expect("same IPv4 prefix target")
                .index,
            1
        );

        upstream.targets[1]
            .active_available
            .store(false, Ordering::Release);
        assert_eq!(
            upstream
                .select_target_excluding_observed(&AttemptedTargets::default(), affinity_ip, None,)
                .expect("deterministic health rehash target")
                .index,
            0,
            "the second Nginx hash maps the unavailable client to target zero"
        );

        upstream.targets[1]
            .active_available
            .store(true, Ordering::Release);
        let retry_ip = "10.20.3.9".parse::<IpAddr>().expect("retry IPv4");
        let first = upstream
            .select_target_excluding_observed(&AttemptedTargets::default(), retry_ip, None)
            .expect("initial affinity target");
        assert_eq!(first.index, 0);
        let mut attempted = AttemptedTargets::default();
        attempted.insert(first.index);
        assert_eq!(
            upstream
                .select_target_excluding_observed(&attempted, retry_ip, None)
                .expect("rehash excludes attempted target")
                .index,
            1
        );
    }

    #[test]
    fn ip_hash_keeps_primary_authoritative_and_uses_backup_when_unavailable() {
        let mut upstream = test_weighted_upstream(&[1, 1_000], 1, 1_000);
        upstream.load_balancing = UpstreamLoadBalancingStrategy::IpHash;
        upstream.targets[1].backup = true;
        let client_ip = "2001:db8::42".parse::<IpAddr>().expect("client IPv6");
        assert_eq!(
            upstream
                .select_target_excluding_observed(&AttemptedTargets::default(), client_ip, None,)
                .expect("primary affinity target")
                .index,
            0
        );
        upstream.targets[0]
            .active_available
            .store(false, Ordering::Release);
        assert_eq!(
            upstream
                .select_target_excluding_observed(&AttemptedTargets::default(), client_ip, None,)
                .expect("backup affinity target")
                .index,
            1
        );
    }

    #[test]
    fn ip_hash_mapping_is_stable_across_fresh_generations() {
        let mut first_generation = test_weighted_upstream(&[2, 5, 1], 1, 1_000);
        first_generation.load_balancing = UpstreamLoadBalancingStrategy::IpHash;
        let mut second_generation = test_weighted_upstream(&[2, 5, 1], 1, 1_000);
        second_generation.load_balancing = UpstreamLoadBalancingStrategy::IpHash;
        let client_ip = "2001:db8:1234::9"
            .parse::<IpAddr>()
            .expect("stable client IPv6");

        let first = first_generation
            .select_target_excluding_observed(&AttemptedTargets::default(), client_ip, None)
            .expect("first generation target");
        let second = second_generation
            .select_target_excluding_observed(&AttemptedTargets::default(), client_ip, None)
            .expect("second generation target");
        assert_eq!(first.index, second.index);
    }

    #[test]
    fn weighted_load_comparison_and_activity_lease_are_overflow_safe() {
        assert_eq!(
            super::weighted_load_cmp(usize::MAX, 1_000, usize::MAX, 999),
            std::cmp::Ordering::Less
        );

        let counter = Arc::new(AtomicUsize::new(0));
        let lease = TargetActivityLease::claim(&counter);
        assert_eq!(counter.load(Ordering::Acquire), 1);
        drop(lease);
        assert_eq!(counter.load(Ordering::Acquire), 0);

        counter.store(usize::MAX, Ordering::Release);
        let saturated = TargetActivityLease::claim(&counter);
        assert!(!saturated.acquired);
        drop(saturated);
        assert_eq!(counter.load(Ordering::Acquire), usize::MAX);

        counter.store(0, Ordering::Release);
        drop(TargetActivityLease {
            counter: counter.clone(),
            acquired: true,
        });
        assert_eq!(counter.load(Ordering::Acquire), 0);
    }

    #[test]
    fn slow_start_effective_weight_is_monotonic_bounded_and_restartable() {
        let mut upstream = test_weighted_upstream(&[10], 1, 1_000);
        let target = &mut upstream.targets[0];
        target.slow_start_duration_ms = 1_000;
        target.start_slow_start(100);

        assert_eq!(target.effective_weight(100), 1);
        assert_eq!(target.effective_weight(599), 4);
        assert_eq!(target.effective_weight(600), 5);
        assert_eq!(target.effective_weight(1_099), 9);
        assert_eq!(target.effective_weight(1_100), 10);
        assert_eq!(target.slow_start_started_ms.load(Ordering::Acquire), 0);
        assert_eq!(
            target.ramped_weight(1_100, 1_000),
            1,
            "a concurrently restarted ramp is recomputed instead of using stale nominal weight"
        );

        target.start_slow_start(2_000);
        assert_eq!(target.effective_weight(2_000), 1);
        assert_eq!(target.effective_weight(2_500), 5);
    }

    #[test]
    fn least_connections_uses_slow_start_effective_weight() {
        let mut upstream = test_weighted_upstream(&[10, 1], 1, 1_000);
        upstream.load_balancing = UpstreamLoadBalancingStrategy::LeastConnections;
        upstream.targets[0].slow_start_duration_ms = 1_000;
        upstream.targets[0].start_slow_start(100);
        upstream.targets[0]
            .active_requests
            .store(1, Ordering::Release);
        upstream.targets[1]
            .active_requests
            .store(1, Ordering::Release);
        upstream.cursor.store(1, Ordering::Release);

        assert_eq!(
            upstream
                .select_target()
                .expect("recovery-weight tie remains selectable")
                .index,
            1,
            "effective weights are 1:1 during the first slow-start slot"
        );
        upstream.targets[0]
            .slow_start_started_ms
            .store(0, Ordering::Release);
        assert_eq!(
            upstream
                .select_target()
                .expect("nominal least-load target remains selectable")
                .index,
            0,
            "after slow start, 1/10 is below 1/1"
        );
    }

    #[tokio::test]
    async fn active_and_passive_recovery_start_slow_start_only_when_eligible() {
        let mut active = test_weighted_upstream(&[10], 1, 20);
        active.targets[0].slow_start_duration_ms = 1_000;
        active.active_health = Some(ActiveHealthPolicy {
            method: UpstreamActiveHealthMethod::Get,
            interval: Duration::from_secs(1),
            timeout: Duration::from_millis(100),
            unhealthy_threshold: 1,
            healthy_threshold: 1,
            success_status_min: 200,
            success_status_max: 399,
            max_response_body_bytes: 64,
        });
        assert_eq!(
            active.record_active_health(0, false),
            ActiveHealthTransition::BecameUnhealthy
        );
        assert_eq!(
            active.targets[0]
                .slow_start_started_ms
                .load(Ordering::Acquire),
            0
        );
        assert_eq!(
            active.record_active_health(0, true),
            ActiveHealthTransition::BecameHealthy
        );
        assert_ne!(
            active.targets[0]
                .slow_start_started_ms
                .load(Ordering::Acquire),
            0
        );
        assert_eq!(active.targets[0].effective_weight(active.now_ms()), 1);

        let mut passive = test_weighted_upstream(&[10], 1, 20);
        passive.targets[0].slow_start_duration_ms = 1_000;
        passive.active_health = Some(ActiveHealthPolicy {
            method: UpstreamActiveHealthMethod::Get,
            interval: Duration::from_secs(1),
            timeout: Duration::from_millis(100),
            unhealthy_threshold: 1,
            healthy_threshold: 1,
            success_status_min: 200,
            success_status_max: 399,
            max_response_body_bytes: 64,
        });
        let failed = passive.select_target().expect("initial target selection");
        passive.record_failure(failed);
        assert_eq!(
            passive.record_active_health(0, false),
            ActiveHealthTransition::BecameUnhealthy
        );
        assert_eq!(
            passive.record_active_health(0, true),
            ActiveHealthTransition::BecameHealthy
        );
        assert_eq!(
            passive.targets[0]
                .slow_start_started_ms
                .load(Ordering::Acquire),
            0,
            "active recovery must not consume slow start while passive ejection remains"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
        let probe = passive.select_target().expect("half-open probe selection");
        assert!(probe.probe);
        passive.record_success(probe);
        assert_ne!(
            passive.targets[0]
                .slow_start_started_ms
                .load(Ordering::Acquire),
            0
        );
        assert_eq!(passive.targets[0].effective_weight(passive.now_ms()), 1);

        let failed_again = passive.select_target().expect("recovered target selection");
        passive.record_failure(failed_again);
        assert_eq!(
            passive.targets[0]
                .slow_start_started_ms
                .load(Ordering::Acquire),
            0
        );
    }

    #[tokio::test]
    async fn ejection_allows_one_half_open_probe_and_recovers_or_restarts_deadline() {
        let upstream = test_weighted_upstream(&[100], 1, 20);
        assert_eq!(upstream.aggregate_target_health(), [1, 0, 0, 0]);
        let first = upstream
            .select_target()
            .expect("healthy target is selected");
        assert!(!first.probe);
        upstream.record_failure(first);
        assert_eq!(upstream.aggregate_target_health(), [0, 1, 0, 0]);
        assert!(upstream.select_target().is_none());

        tokio::time::sleep(Duration::from_millis(25)).await;
        let failed_probe = upstream
            .select_target()
            .expect("one probe is selected after ejection");
        assert!(failed_probe.probe);
        assert_eq!(upstream.aggregate_target_health(), [0, 0, 1, 0]);
        assert!(upstream.select_target().is_none());
        upstream.record_failure(failed_probe);
        assert_eq!(upstream.aggregate_target_health(), [0, 1, 0, 0]);
        assert!(upstream.select_target().is_none());

        tokio::time::sleep(Duration::from_millis(25)).await;
        let successful_probe = upstream
            .select_target()
            .expect("new probe is selected after the restarted deadline");
        assert!(successful_probe.probe);
        upstream.record_success(successful_probe);
        assert_eq!(upstream.aggregate_target_health(), [1, 0, 0, 0]);
        let normal = upstream
            .select_target()
            .expect("successful probe restores normal selection");
        assert!(!normal.probe);
    }

    #[test]
    fn weighted_selection_excludes_actively_unavailable_target() {
        let upstream = test_weighted_upstream(&[100, 1], 3, 1_000);
        upstream.targets[0]
            .active_available
            .store(false, std::sync::atomic::Ordering::Release);
        for _ in 0..4 {
            assert_eq!(
                upstream
                    .select_target()
                    .expect("eligible target remains")
                    .index,
                1
            );
        }
    }

    #[test]
    fn ejected_target_is_skipped_while_healthy_alternative_remains() {
        let upstream = test_upstream(2, 1, 1_000);
        let failed = upstream.select_target().expect("first target selected");
        assert_eq!(failed.index, 0);
        upstream.record_failure(failed);

        for _ in 0..4 {
            let selected = upstream
                .select_target()
                .expect("healthy alternative remains selectable");
            assert_eq!(selected.index, 1);
            upstream.record_success(selected);
        }
    }

    #[test]
    fn stale_success_cannot_clear_a_newer_ejection() {
        let upstream = test_upstream(1, 1, 1_000);
        let failing = upstream.select_target().expect("first concurrent request");
        let stale_success = upstream.select_target().expect("second concurrent request");
        upstream.record_failure(failing);
        upstream.record_success(stale_success);
        assert!(
            upstream.select_target().is_none(),
            "success selected before ejection cannot clear the newer state"
        );
    }

    #[test]
    fn active_thresholds_gate_selection_without_clearing_passive_ejection() {
        let mut upstream = test_upstream(1, 1, 1_000);
        upstream.active_health = Some(ActiveHealthPolicy {
            method: UpstreamActiveHealthMethod::Get,
            interval: Duration::from_secs(1),
            timeout: Duration::from_millis(100),
            unhealthy_threshold: 2,
            healthy_threshold: 2,
            success_status_min: 200,
            success_status_max: 399,
            max_response_body_bytes: 64,
        });

        assert_eq!(
            upstream.record_active_health(0, false),
            ActiveHealthTransition::Unchanged
        );
        assert!(upstream.select_target().is_some());
        assert_eq!(
            upstream.record_active_health(0, false),
            ActiveHealthTransition::BecameUnhealthy
        );
        assert_eq!(upstream.aggregate_target_health(), [0, 0, 0, 1]);
        assert!(upstream.select_target().is_none());
        assert_eq!(
            upstream.record_active_health(0, true),
            ActiveHealthTransition::Unchanged
        );
        assert!(upstream.select_target().is_none());
        assert_eq!(
            upstream.record_active_health(0, true),
            ActiveHealthTransition::BecameHealthy
        );
        assert_eq!(upstream.aggregate_target_health(), [1, 0, 0, 0]);

        let selected = upstream
            .select_target()
            .expect("active recovery restores selection");
        upstream.record_failure(selected);
        assert!(upstream.select_target().is_none());
        assert_eq!(
            upstream.record_active_health(0, true),
            ActiveHealthTransition::Unchanged
        );
        assert!(
            upstream.select_target().is_none(),
            "active success must not clear passive ejection"
        );
    }

    #[test]
    fn business_success_cannot_restore_an_actively_unhealthy_target() {
        let mut upstream = test_upstream(1, 1, 1_000);
        upstream.active_health = Some(ActiveHealthPolicy {
            method: UpstreamActiveHealthMethod::Get,
            interval: Duration::from_secs(1),
            timeout: Duration::from_millis(100),
            unhealthy_threshold: 1,
            healthy_threshold: 1,
            success_status_min: 200,
            success_status_max: 399,
            max_response_body_bytes: 64,
        });
        let selected = upstream
            .select_target()
            .expect("target starts actively available");
        assert_eq!(
            upstream.record_active_health(0, false),
            ActiveHealthTransition::BecameUnhealthy
        );
        upstream.record_success(selected);
        assert!(
            upstream.select_target().is_none(),
            "business success must not clear active health state"
        );
    }
}
