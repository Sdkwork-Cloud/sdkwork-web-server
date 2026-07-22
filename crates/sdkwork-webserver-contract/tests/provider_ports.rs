use std::collections::VecDeque;

use async_trait::async_trait;
use sdkwork_webserver_contract::{
    OpenedWebsiteContent, WebsiteContentRange, WebsiteProviderContentHandle,
    WebsiteProviderContentStream, WebsiteProviderError, WebsiteProviderErrorKind,
    WebsiteProviderPageSize, WebsiteProviderResult,
};

struct FakeContentStream {
    chunks: VecDeque<Vec<u8>>,
}

#[async_trait]
impl WebsiteProviderContentStream for FakeContentStream {
    async fn next_chunk(&mut self) -> WebsiteProviderResult<Option<Vec<u8>>> {
        Ok(self.chunks.pop_front())
    }
}

#[test]
fn page_size_is_bounded_at_the_provider_contract_boundary() {
    assert_eq!(WebsiteProviderPageSize::DEFAULT.get(), 20);
    assert_eq!(WebsiteProviderPageSize::try_from(200).unwrap().get(), 200);
    assert!(WebsiteProviderPageSize::try_from(0).is_err());
    assert!(WebsiteProviderPageSize::try_from(201).is_err());
    assert!(serde_json::from_str::<WebsiteProviderPageSize>("201").is_err());
}

#[test]
fn content_handles_are_bounded_and_redacted_from_debug_output() {
    let handle = WebsiteProviderContentHandle::new("provider:opaque-content-version").unwrap();
    assert_eq!(handle.as_str(), "provider:opaque-content-version");
    let debug = format!("{handle:?}");
    assert!(debug.contains("[REDACTED]"));
    assert!(!debug.contains("opaque-content-version"));
    assert!(WebsiteProviderContentHandle::new("").is_err());
    assert!(WebsiteProviderContentHandle::new("x".repeat(513)).is_err());
}

#[test]
fn provider_errors_expose_typed_retry_policy_without_origin_details() {
    let error =
        WebsiteProviderError::with_retry_after(WebsiteProviderErrorKind::RateLimited, 1_500);
    let json = serde_json::to_value(&error).unwrap();
    assert_eq!(json["kind"], "RATE_LIMITED");
    assert_eq!(json["retryAfterMs"], 1_500);
    assert!(json.get("message").is_none());
    assert!(json.get("url").is_none());
}

#[tokio::test]
async fn content_stream_contract_is_incremental() {
    let mut stream = FakeContentStream {
        chunks: VecDeque::from([b"first".to_vec(), b"second".to_vec()]),
    };
    assert_eq!(stream.next_chunk().await.unwrap(), Some(b"first".to_vec()));
    assert_eq!(stream.next_chunk().await.unwrap(), Some(b"second".to_vec()));
    assert_eq!(stream.next_chunk().await.unwrap(), None);
}

#[test]
fn opened_content_carries_exact_http_range_evidence() {
    let opened = OpenedWebsiteContent {
        stream: Box::new(FakeContentStream {
            chunks: VecDeque::new(),
        }),
        content_length: 100,
        content_range: Some(WebsiteContentRange {
            start: 100,
            end_inclusive: 199,
            complete_length: 1_000,
        }),
    };
    assert_eq!(opened.content_length, 100);
    assert_eq!(
        opened.content_range,
        Some(WebsiteContentRange {
            start: 100,
            end_inclusive: 199,
            complete_length: 1_000,
        })
    );
}
