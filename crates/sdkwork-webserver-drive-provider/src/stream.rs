use async_trait::async_trait;
use sdkwork_webserver_contract::provider::{WebsiteProviderContentStream, WebsiteProviderResult};

pub(crate) struct BoundedDriveContentStream {
    content: Option<Vec<u8>>,
}

impl BoundedDriveContentStream {
    pub(crate) fn new(content: Vec<u8>) -> Self {
        Self {
            content: Some(content),
        }
    }
}

#[async_trait]
impl WebsiteProviderContentStream for BoundedDriveContentStream {
    async fn next_chunk(&mut self) -> WebsiteProviderResult<Option<Vec<u8>>> {
        Ok(self.content.take())
    }
}
