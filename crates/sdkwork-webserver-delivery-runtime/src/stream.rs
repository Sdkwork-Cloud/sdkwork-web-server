use async_trait::async_trait;
use sdkwork_webserver_contract::provider::{
    WebsiteProviderContentStream, WebsiteProviderError, WebsiteProviderErrorKind,
    WebsiteProviderResult,
};
use std::time::Duration;
use tokio::sync::OwnedSemaphorePermit;
use tokio::time::timeout;

pub(crate) struct AdmittedProviderContentStream {
    inner: Box<dyn WebsiteProviderContentStream>,
    permit: Option<OwnedSemaphorePermit>,
}

impl AdmittedProviderContentStream {
    pub(crate) fn new(
        inner: Box<dyn WebsiteProviderContentStream>,
        permit: OwnedSemaphorePermit,
    ) -> Self {
        Self {
            inner,
            permit: Some(permit),
        }
    }
}

#[async_trait]
impl WebsiteProviderContentStream for AdmittedProviderContentStream {
    async fn next_chunk(&mut self) -> WebsiteProviderResult<Option<Vec<u8>>> {
        let result = self.inner.next_chunk().await;
        if !matches!(result, Ok(Some(_))) {
            self.permit.take();
        }
        result
    }
}

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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sdkwork_webserver_contract::provider::{WebsiteProviderError, WebsiteProviderErrorKind};
    use tokio::sync::Semaphore;

    use super::*;

    struct TerminalStream {
        result: Option<WebsiteProviderResult<Option<Vec<u8>>>>,
    }

    #[async_trait]
    impl WebsiteProviderContentStream for TerminalStream {
        async fn next_chunk(&mut self) -> WebsiteProviderResult<Option<Vec<u8>>> {
            self.result.take().expect("test stream is polled once")
        }
    }

    fn admitted_stream(
        semaphore: &Arc<Semaphore>,
        result: WebsiteProviderResult<Option<Vec<u8>>>,
    ) -> AdmittedProviderContentStream {
        let permit = Arc::clone(semaphore)
            .try_acquire_many_owned(3)
            .expect("test admission must succeed");
        AdmittedProviderContentStream::new(
            Box::new(TerminalStream {
                result: Some(result),
            }),
            permit,
        )
    }

    #[tokio::test]
    async fn admission_releases_on_completion_error_and_drop() {
        let semaphore = Arc::new(Semaphore::new(3));

        let mut completed = admitted_stream(&semaphore, Ok(None));
        assert_eq!(semaphore.available_permits(), 0);
        assert!(completed.next_chunk().await.unwrap().is_none());
        assert_eq!(semaphore.available_permits(), 3);

        let mut failed = admitted_stream(
            &semaphore,
            Err(WebsiteProviderError::new(
                WebsiteProviderErrorKind::Unavailable,
            )),
        );
        assert!(failed.next_chunk().await.is_err());
        assert_eq!(semaphore.available_permits(), 3);

        let cancelled = admitted_stream(&semaphore, Ok(Some(vec![1])));
        assert_eq!(semaphore.available_permits(), 0);
        drop(cancelled);
        assert_eq!(semaphore.available_permits(), 3);
    }
}
