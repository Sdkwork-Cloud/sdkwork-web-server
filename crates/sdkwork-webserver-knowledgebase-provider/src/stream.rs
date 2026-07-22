use async_trait::async_trait;
use sdkwork_webserver_contract::provider::{WebsiteProviderContentStream, WebsiteProviderResult};

pub(crate) struct BoundedWikiContentStream {
    content: Option<Vec<u8>>,
}

impl BoundedWikiContentStream {
    pub(crate) fn new(content: Vec<u8>) -> Self {
        Self {
            content: Some(content),
        }
    }
}

#[async_trait]
impl WebsiteProviderContentStream for BoundedWikiContentStream {
    async fn next_chunk(&mut self) -> WebsiteProviderResult<Option<Vec<u8>>> {
        Ok(self.content.take())
    }
}
