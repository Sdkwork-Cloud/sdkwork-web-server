use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WebsiteProviderErrorKind {
    NotFound,
    NotPublic,
    NotModified,
    PreconditionFailed,
    RangeNotSatisfiable,
    InvalidPath,
    Revoked,
    RateLimited,
    DeadlineExceeded,
    Unavailable,
    ContractMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq, Error, Serialize, Deserialize)]
#[error("website provider failed with {kind:?}")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WebsiteProviderError {
    pub kind: WebsiteProviderErrorKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_after_ms: Option<u64>,
}

impl WebsiteProviderError {
    pub fn new(kind: WebsiteProviderErrorKind) -> Self {
        Self {
            kind,
            retry_after_ms: None,
        }
    }

    pub fn with_retry_after(kind: WebsiteProviderErrorKind, retry_after_ms: u64) -> Self {
        Self {
            kind,
            retry_after_ms: Some(retry_after_ms),
        }
    }
}

pub type WebsiteProviderResult<T> = Result<T, WebsiteProviderError>;
