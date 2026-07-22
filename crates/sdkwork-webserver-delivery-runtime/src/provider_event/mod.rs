use sdkwork_utils_rust::sha256_hash;

mod checkpoint;
mod model;
mod processor;
mod reconciliation;

pub(super) const PROVIDER_EVENT_STREAM_SHARDS: usize = 64;

pub(super) fn provider_event_stream_shard(stream_id: &str) -> usize {
    let digest = sha256_hash(stream_id.as_bytes());
    let prefix = digest
        .get(..2)
        .and_then(|value| u8::from_str_radix(value, 16).ok())
        .unwrap_or_default();
    usize::from(prefix) % PROVIDER_EVENT_STREAM_SHARDS
}

pub use checkpoint::{
    FileWebsiteProviderEventCheckpointStore, WebsiteProviderEventCheckpoint,
    WebsiteProviderEventCheckpointError, WebsiteProviderEventCheckpointStore,
};
pub use model::{
    parse_website_provider_event, WebsiteProviderEvent, WebsiteProviderEventInvalidation,
    WebsiteProviderEventInvalidationKind, WebsiteProviderEventInvalidationPriority,
    WebsiteProviderEventOrdering, WebsiteProviderEventParseError, WebsiteProviderEventScope,
    WebsiteProviderEventSource, MAXIMUM_PROVIDER_EVENT_BYTES,
};
pub use processor::{
    CachelessWebsiteProviderEventInvalidator, WebsiteProviderEventInvalidator,
    WebsiteProviderEventProcessError, WebsiteProviderEventProcessOutcome,
    WebsiteProviderEventProcessor, WebsiteProviderEventReconciler,
};
pub use reconciliation::WebsiteRuntimeSetProviderEventReconciler;
