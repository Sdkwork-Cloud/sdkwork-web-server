use std::collections::{HashMap, HashSet};

use sdkwork_webserver_contract::provider::WebsiteProviderResult;
use sdkwork_webserver_core::website_runtime::WebsiteProviderType;
use tokio::{
    sync::watch,
    time::Instant,
};

use crate::{
    WebsiteProviderEventInvalidation, WebsiteProviderEventInvalidationKind,
    WebsiteProviderEventSource,
};

use super::model::{
    CachedResolution, ProviderCacheIdentity, ResolutionCacheKey, ResolutionCachePolicy,
};

pub(super) type FlightResult = WebsiteProviderResult<CachedResolution>;

pub(super) struct FlightStart {
    pub(super) sender: watch::Sender<Option<FlightResult>>,
    pub(super) receiver: watch::Receiver<Option<FlightResult>>,
    pub(super) provider_epoch: u64,
}

pub(super) enum CacheLookup {
    Fresh(CachedResolution),
    Stale {
        value: CachedResolution,
        refresh: Option<FlightStart>,
    },
    Wait(watch::Receiver<Option<FlightResult>>),
    Start(FlightStart),
    Bypass,
}

struct CacheEntry {
    value: CachedResolution,
    fresh_until: Instant,
    stale_until: Instant,
    access_sequence: u64,
}

#[derive(Default)]
pub(super) struct CacheState {
    entries: HashMap<ResolutionCacheKey, CacheEntry>,
    in_flight: HashMap<ResolutionCacheKey, watch::Receiver<Option<FlightResult>>>,
    provider_epochs: HashMap<ProviderCacheIdentity, u64>,
    access_sequence: u64,
}

impl CacheState {
    pub(super) fn lookup_or_start(
        &mut self,
        key: &ResolutionCacheKey,
        maximum_entries: usize,
        now: Instant,
    ) -> CacheLookup {
        self.access_sequence = self.access_sequence.wrapping_add(1);
        let access_sequence = self.access_sequence;
        if let Some(entry) = self.entries.get_mut(key) {
            entry.access_sequence = access_sequence;
            if now < entry.fresh_until {
                return CacheLookup::Fresh(entry.value.clone());
            }
            if now < entry.stale_until {
                let value = entry.value.clone();
                let refresh = if self.in_flight.contains_key(key) {
                    None
                } else {
                    self.start_flight(key, maximum_entries)
                };
                return CacheLookup::Stale { value, refresh };
            }
        }
        self.entries.remove(key);
        if let Some(receiver) = self.in_flight.get(key) {
            return CacheLookup::Wait(receiver.clone());
        }
        self.start_flight(key, maximum_entries)
            .map_or(CacheLookup::Bypass, CacheLookup::Start)
    }

    fn start_flight(
        &mut self,
        key: &ResolutionCacheKey,
        maximum_entries: usize,
    ) -> Option<FlightStart> {
        if self.in_flight.len() >= maximum_entries {
            return None;
        }
        let provider_epoch = *self
            .provider_epochs
            .entry(key.provider().clone())
            .or_insert(0);
        let (sender, receiver) = watch::channel(None);
        self.in_flight.insert(key.clone(), receiver.clone());
        Some(FlightStart {
            sender,
            receiver,
            provider_epoch,
        })
    }

    pub(super) fn provider_epoch_matches(
        &self,
        key: &ResolutionCacheKey,
        expected: u64,
    ) -> bool {
        self.provider_epochs
            .get(key.provider())
            .copied()
            .unwrap_or_default()
            == expected
    }

    pub(super) fn remove_flight(&mut self, key: &ResolutionCacheKey) {
        self.in_flight.remove(key);
    }

    pub(super) fn insert(
        &mut self,
        key: ResolutionCacheKey,
        value: CachedResolution,
        policy: ResolutionCachePolicy,
        maximum_entries: usize,
        now: Instant,
    ) -> bool {
        let (ttl, stale) = match &value {
            CachedResolution::Negative => (policy.negative_ttl, std::time::Duration::ZERO),
            value if value.is_positive_cacheable() => {
                (policy.metadata_ttl, policy.stale_while_revalidate)
            }
            _ => return false,
        };
        if ttl.is_zero() {
            return false;
        }
        let evicted = if self.entries.len() >= maximum_entries
            && !self.entries.contains_key(&key)
        {
            self.evict_oldest()
        } else {
            false
        };
        self.access_sequence = self.access_sequence.wrapping_add(1);
        let fresh_until = now + ttl;
        self.entries.insert(
            key,
            CacheEntry {
                value,
                fresh_until,
                stale_until: fresh_until + stale,
                access_sequence: self.access_sequence,
            },
        );
        evicted
    }

    fn evict_oldest(&mut self) -> bool {
        let Some(key) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.access_sequence)
            .map(|(key, _)| key.clone())
        else {
            return false;
        };
        self.entries.remove(&key);
        true
    }

    pub(super) fn invalidate(
        &mut self,
        invalidations: &[WebsiteProviderEventInvalidation],
    ) -> usize {
        let mut provider_wide = HashSet::new();
        let mut exact_routes = HashSet::new();
        let mut affected_providers = HashSet::new();
        for invalidation in invalidations {
            let provider = ProviderCacheIdentity {
                provider_type: invalidation.provider_type,
                provider_resource_uuid: invalidation.provider_resource_uuid.clone(),
            };
            affected_providers.insert(provider.clone());
            match &invalidation.kind {
                WebsiteProviderEventInvalidationKind::Route { path } => {
                    exact_routes.insert((provider, path.clone()));
                }
                WebsiteProviderEventInvalidationKind::Provider
                | WebsiteProviderEventInvalidationKind::Navigation
                | WebsiteProviderEventInvalidationKind::Search => {
                    provider_wide.insert(provider);
                }
            }
        }
        for provider in &affected_providers {
            if let Some(epoch) = self.provider_epochs.get_mut(provider) {
                *epoch = epoch.wrapping_add(1);
            }
        }
        let before = self.entries.len();
        self.entries.retain(|key, _| {
            !provider_wide.contains(key.provider())
                && !exact_routes.contains(&(key.provider().clone(), key.path().to_owned()))
        });
        self.cleanup_provider_epochs();
        before - self.entries.len()
    }

    pub(super) fn mark_uncertain(&mut self, source: WebsiteProviderEventSource) -> usize {
        let provider_type = match source {
            WebsiteProviderEventSource::Drive => WebsiteProviderType::Drive,
            WebsiteProviderEventSource::Knowledgebase => WebsiteProviderType::Knowledgebase,
        };
        for (provider, epoch) in &mut self.provider_epochs {
            if provider.provider_type == provider_type {
                *epoch = epoch.wrapping_add(1);
            }
        }
        let before = self.entries.len();
        self.entries
            .retain(|key, _| key.provider().provider_type != provider_type);
        self.cleanup_provider_epochs();
        before - self.entries.len()
    }

    pub(super) fn cleanup_provider_epochs(&mut self) {
        let active = self
            .entries
            .keys()
            .chain(self.in_flight.keys())
            .map(|key| key.provider().clone())
            .collect::<HashSet<_>>();
        self.provider_epochs
            .retain(|provider, _| active.contains(provider));
    }

    pub(super) fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub(super) fn in_flight_count(&self) -> usize {
        self.in_flight.len()
    }
}
