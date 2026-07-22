use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::Mutex;

use super::{
    parse_website_provider_event, provider_event_stream_shard, WebsiteProviderEvent,
    WebsiteProviderEventCheckpoint, WebsiteProviderEventCheckpointError,
    WebsiteProviderEventCheckpointStore, WebsiteProviderEventInvalidation,
    WebsiteProviderEventOrdering, WebsiteProviderEventParseError, WebsiteProviderEventScope,
    PROVIDER_EVENT_STREAM_SHARDS,
};

#[async_trait]
pub trait WebsiteProviderEventInvalidator: Send + Sync {
    async fn mark_uncertain(&self, scope: &WebsiteProviderEventScope) -> Result<(), String>;

    async fn invalidate(
        &self,
        invalidations: &[WebsiteProviderEventInvalidation],
    ) -> Result<(), String>;
}

#[async_trait]
pub trait WebsiteProviderEventReconciler: Send + Sync {
    async fn reconcile(&self, event: &WebsiteProviderEvent) -> Result<(), String>;
}

pub struct CachelessWebsiteProviderEventInvalidator;

#[async_trait]
impl WebsiteProviderEventInvalidator for CachelessWebsiteProviderEventInvalidator {
    async fn mark_uncertain(&self, _scope: &WebsiteProviderEventScope) -> Result<(), String> {
        Ok(())
    }

    async fn invalidate(
        &self,
        _invalidations: &[WebsiteProviderEventInvalidation],
    ) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebsiteProviderEventProcessOutcome {
    Applied,
    ReconciledAndApplied,
    DuplicateIgnored,
    StaleIgnored,
}

#[derive(Debug, Error)]
pub enum WebsiteProviderEventProcessError {
    #[error(transparent)]
    Parse(#[from] WebsiteProviderEventParseError),
    #[error(transparent)]
    Checkpoint(#[from] WebsiteProviderEventCheckpointError),
    #[error("provider event stream contains a conflicting event identity or sequence")]
    ContractConflict,
    #[error("provider event uncertainty invalidation failed: {0}")]
    Uncertainty(String),
    #[error("provider event reconciliation failed: {0}")]
    Reconciliation(String),
    #[error("provider event cache invalidation failed: {0}")]
    Invalidation(String),
}

pub struct WebsiteProviderEventProcessor {
    checkpoints: Arc<dyn WebsiteProviderEventCheckpointStore>,
    invalidator: Arc<dyn WebsiteProviderEventInvalidator>,
    reconciler: Arc<dyn WebsiteProviderEventReconciler>,
    processing: [Mutex<()>; PROVIDER_EVENT_STREAM_SHARDS],
}

impl WebsiteProviderEventProcessor {
    pub fn new(
        checkpoints: Arc<dyn WebsiteProviderEventCheckpointStore>,
        invalidator: Arc<dyn WebsiteProviderEventInvalidator>,
        reconciler: Arc<dyn WebsiteProviderEventReconciler>,
    ) -> Self {
        Self {
            checkpoints,
            invalidator,
            reconciler,
            processing: std::array::from_fn(|_| Mutex::new(())),
        }
    }

    pub async fn process(
        &self,
        body: &[u8],
    ) -> Result<WebsiteProviderEventProcessOutcome, WebsiteProviderEventProcessError> {
        let event = parse_website_provider_event(body)?;
        self.process_event(event).await
    }

    pub async fn process_event(
        &self,
        event: WebsiteProviderEvent,
    ) -> Result<WebsiteProviderEventProcessOutcome, WebsiteProviderEventProcessError> {
        let processing_shard = provider_event_stream_shard(&event.scope.stream_id);
        let _processing = self.processing[processing_shard].lock().await;
        let mut checkpoint = self.checkpoints.load(&event.scope.stream_id).await?;
        let mut reconciled = false;

        if checkpoint
            .as_ref()
            .is_some_and(|value| value.is_uncertain())
        {
            self.invalidator
                .mark_uncertain(&event.scope)
                .await
                .map_err(WebsiteProviderEventProcessError::Uncertainty)?;
            self.reconciler
                .reconcile(&event)
                .await
                .map_err(WebsiteProviderEventProcessError::Reconciliation)?;
            let current = checkpoint.as_mut().expect("checked checkpoint presence");
            current.set_uncertain(false);
            self.checkpoints.save(current).await?;
            reconciled = true;
        }

        if let Some(current) = checkpoint.as_ref() {
            if let Some((sequence_no, payload_sha256)) = current.recent_event(&event.id) {
                if sequence_no == event.sequence_no && payload_sha256 == event.payload_sha256 {
                    return Ok(WebsiteProviderEventProcessOutcome::DuplicateIgnored);
                }
                self.persist_conflict(current.clone(), &event.scope).await?;
                return Err(WebsiteProviderEventProcessError::ContractConflict);
            }
            if event.sequence_no < current.last_sequence_no() {
                return Ok(WebsiteProviderEventProcessOutcome::StaleIgnored);
            }
            if event.sequence_no == current.last_sequence_no() {
                self.persist_conflict(current.clone(), &event.scope).await?;
                return Err(WebsiteProviderEventProcessError::ContractConflict);
            }
        }

        let contiguous_gap = checkpoint.as_ref().is_some_and(|current| {
            event.ordering == WebsiteProviderEventOrdering::Contiguous
                && current.last_sequence_no() > 0
                && event.sequence_no != current.last_sequence_no().saturating_add(1)
        });
        if checkpoint.is_none() || contiguous_gap {
            let mut uncertain = checkpoint.unwrap_or_else(|| {
                WebsiteProviderEventCheckpoint::uncertain(event.scope.stream_id.clone())
            });
            uncertain.set_uncertain(true);
            self.checkpoints.save(&uncertain).await?;
            self.invalidator
                .mark_uncertain(&event.scope)
                .await
                .map_err(WebsiteProviderEventProcessError::Uncertainty)?;
            self.reconciler
                .reconcile(&event)
                .await
                .map_err(WebsiteProviderEventProcessError::Reconciliation)?;
            uncertain.set_uncertain(false);
            self.checkpoints.save(&uncertain).await?;
            checkpoint = Some(uncertain);
            reconciled = true;
        }

        self.invalidator
            .invalidate(&event.invalidations)
            .await
            .map_err(WebsiteProviderEventProcessError::Invalidation)?;
        let mut checkpoint = checkpoint.unwrap_or_else(|| {
            WebsiteProviderEventCheckpoint::uncertain(event.scope.stream_id.clone())
        });
        checkpoint.record(event.id, event.sequence_no, event.payload_sha256);
        self.checkpoints.save(&checkpoint).await?;
        Ok(if reconciled {
            WebsiteProviderEventProcessOutcome::ReconciledAndApplied
        } else {
            WebsiteProviderEventProcessOutcome::Applied
        })
    }

    async fn persist_conflict(
        &self,
        mut checkpoint: WebsiteProviderEventCheckpoint,
        scope: &WebsiteProviderEventScope,
    ) -> Result<(), WebsiteProviderEventProcessError> {
        checkpoint.set_uncertain(true);
        self.checkpoints.save(&checkpoint).await?;
        self.invalidator
            .mark_uncertain(scope)
            .await
            .map_err(WebsiteProviderEventProcessError::Uncertainty)
    }
}
