use sdkwork_webserver_contract::provider::{WebsiteProviderError, WebsiteProviderErrorKind};
use sdkwork_webserver_core::website_runtime::{
    WebsiteHandler, WebsiteProviderType, WebsiteRouteSelectionError,
};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum WebsiteDeliveryExecutorConfigError {
    #[error(
        "provider buffered-content budget {configured_bytes} is outside 1..={maximum_bytes} bytes"
    )]
    InvalidBufferedContentBudget {
        configured_bytes: usize,
        maximum_bytes: usize,
    },
    #[error(
        "provider resolution cache capacity {configured_entries} is outside 1..={maximum_entries} entries"
    )]
    InvalidProviderResolutionCacheCapacity {
        configured_entries: usize,
        maximum_entries: usize,
    },
}

#[derive(Debug, Error)]
pub enum WebsiteDeliveryError {
    #[error("website runtime set is unavailable")]
    RuntimeUnavailable,
    #[error(transparent)]
    RouteSelection(#[from] WebsiteRouteSelectionError),
    #[error("website delivery request identity is invalid")]
    InvalidRequestIdentity,
    #[error("no {capability} provider is registered for {provider_type:?}")]
    ProviderNotRegistered {
        provider_type: WebsiteProviderType,
        capability: &'static str,
    },
    #[error("handler {handler:?} is incompatible with the selected provider registry entry")]
    HandlerNotSupported { handler: WebsiteHandler },
    #[error("the selected representation does not support byte ranges")]
    RangeNotSupported,
    #[error(
        "declared content length {declared_bytes} exceeds the runtime maximum {maximum_bytes}"
    )]
    ContentTooLarge {
        declared_bytes: u64,
        maximum_bytes: u64,
    },
    #[error(transparent)]
    Provider(#[from] WebsiteProviderError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum WebsiteProviderRegistryError {
    #[error("a {capability} provider is already registered for {provider_type:?}")]
    DuplicateProvider {
        provider_type: WebsiteProviderType,
        capability: &'static str,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum WebsiteRuntimeProviderValidationError {
    #[error("provider validation concurrency must be greater than zero")]
    InvalidConcurrency,
    #[error("no {capability} provider is registered for {provider_type:?}")]
    ProviderNotRegistered {
        provider_type: WebsiteProviderType,
        capability: &'static str,
    },
    #[error(
        "{provider_type:?} resource {provider_resource_uuid} requests an object limit of {requested_bytes} bytes; provider maximum is {maximum_bytes} bytes"
    )]
    ObjectLimitUnsupported {
        provider_type: WebsiteProviderType,
        provider_resource_uuid: String,
        requested_bytes: u64,
        maximum_bytes: u64,
    },
    #[error(
        "{provider_type:?} resource {provider_resource_uuid} failed activation validation with {kind:?}"
    )]
    Provider {
        provider_type: WebsiteProviderType,
        provider_resource_uuid: String,
        kind: WebsiteProviderErrorKind,
    },
}
