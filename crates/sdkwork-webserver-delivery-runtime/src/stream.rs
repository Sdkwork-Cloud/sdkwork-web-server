use async_trait::async_trait;
use sdkwork_webserver_contract::provider::{
    WebsiteProviderContentStream, WebsiteProviderError, WebsiteProviderErrorKind,
    WebsiteProviderResult,
};
use std::time::Duration;
use tokio::time::timeout;

pub(crate) struct BoundedProviderContentStream {
    inner: Box<dyn WebsiteProviderContentStream>,
    observed_bytes: u64,
    maximum_bytes: u64,
    expected_bytes: Option<u64>,
    chunk_timeout_ms: u64,
    completed: bool,
}

impl BoundedProviderContentStream {
    pub(crate) fn new(
        inner: Box<dyn WebsiteProviderContentStream>,
        maximum_bytes: u64,
        expected_bytes: Option<u64>,
        chunk_timeout_ms: u64,
    ) -> Self {
        Self {
            inner,
            observed_bytes: 0,
            maximum_bytes,
            expected_bytes,
            chunk_timeout_ms,
            completed: false,
        }
    }
}

#[async_trait]
impl WebsiteProviderContentStream for BoundedProviderContentStream {
    async fn next_chunk(&mut self) -> WebsiteProviderResult<Option<Vec<u8>>> {
        if self.completed {
            return Ok(None);
        }
        let next = timeout(
            Duration::from_millis(self.chunk_timeout_ms),
            self.inner.next_chunk(),
        )
        .await
        .map_err(|_| WebsiteProviderError::new(WebsiteProviderErrorKind::DeadlineExceeded))??;
        match next {
            Some(chunk) => {
                let chunk_bytes = u64::try_from(chunk.len()).map_err(|_| contract_mismatch())?;
                self.observed_bytes = self
                    .observed_bytes
                    .checked_add(chunk_bytes)
                    .ok_or_else(contract_mismatch)?;
                if self.observed_bytes > self.maximum_bytes
                    || self
                        .expected_bytes
                        .is_some_and(|expected| self.observed_bytes > expected)
                {
                    return Err(contract_mismatch());
                }
                Ok(Some(chunk))
            }
            None => {
                self.completed = true;
                if self
                    .expected_bytes
                    .is_some_and(|expected| expected != self.observed_bytes)
                {
                    return Err(contract_mismatch());
                }
                Ok(None)
            }
        }
    }
}

fn contract_mismatch() -> WebsiteProviderError {
    WebsiteProviderError::new(WebsiteProviderErrorKind::ContractMismatch)
}
