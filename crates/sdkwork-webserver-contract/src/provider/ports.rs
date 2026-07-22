use async_trait::async_trait;

use super::{
    OpenWebsiteContentRequest, ResolveWebsiteStaticPathRequest, ResolveWebsiteWikiRouteRequest,
    ValidateWebsiteResourceRequest, ValidatedWebsiteResource, WebsiteContentRange,
    WebsiteContentResolution, WebsiteProviderResult, WebsiteWikiCollectionPage,
    WebsiteWikiCollectionRequest, WebsiteWikiRouteResolution,
};

#[async_trait]
pub trait WebsiteProviderContentStream: Send {
    async fn next_chunk(&mut self) -> WebsiteProviderResult<Option<Vec<u8>>>;
}

pub struct OpenedWebsiteContent {
    pub stream: Box<dyn WebsiteProviderContentStream>,
    pub content_length: u64,
    pub content_range: Option<WebsiteContentRange>,
}

#[async_trait]
pub trait WebsiteResourceProvider: Send + Sync {
    fn maximum_content_bytes(&self) -> u64;

    async fn validate_resource(
        &self,
        request: &ValidateWebsiteResourceRequest,
    ) -> WebsiteProviderResult<ValidatedWebsiteResource>;
}

#[async_trait]
pub trait WebsiteStaticContentProvider: WebsiteResourceProvider {
    async fn resolve_static_path(
        &self,
        request: &ResolveWebsiteStaticPathRequest,
    ) -> WebsiteProviderResult<WebsiteContentResolution>;

    async fn open_static_content(
        &self,
        request: &OpenWebsiteContentRequest,
    ) -> WebsiteProviderResult<OpenedWebsiteContent>;
}

#[async_trait]
pub trait WebsiteWikiProvider: WebsiteResourceProvider {
    async fn resolve_wiki_route(
        &self,
        request: &ResolveWebsiteWikiRouteRequest,
    ) -> WebsiteProviderResult<WebsiteWikiRouteResolution>;

    async fn open_wiki_content(
        &self,
        request: &OpenWebsiteContentRequest,
    ) -> WebsiteProviderResult<OpenedWebsiteContent>;

    async fn retrieve_navigation(
        &self,
        request: &WebsiteWikiCollectionRequest,
    ) -> WebsiteProviderResult<WebsiteWikiCollectionPage>;

    async fn search_wiki(
        &self,
        request: &WebsiteWikiCollectionRequest,
    ) -> WebsiteProviderResult<WebsiteWikiCollectionPage>;
}
