use std::sync::Arc;

use async_trait::async_trait;
use sdkwork_knowledgebase_internal_sdk::{
    api::KnowledgebaseInternalWikiApi,
    models::{ResolveWikiRouteRequest, WikiPageListData, WikiPublication, WikiRouteResolution},
    SdkworkError,
};
use sdkwork_webserver_contract::provider::{
    WebsiteProviderError, WebsiteProviderErrorKind, WebsiteProviderResult,
};

#[async_trait]
pub trait KnowledgebaseWikiSdkClient: Send + Sync {
    async fn retrieve_publication(
        &self,
        publication_uuid: &str,
    ) -> Result<WikiPublication, SdkworkError>;

    async fn resolve_route(
        &self,
        publication_uuid: &str,
        request: &ResolveWikiRouteRequest,
    ) -> Result<WikiRouteResolution, SdkworkError>;

    async fn retrieve_content(
        &self,
        publication_uuid: &str,
        content_handle: &str,
    ) -> Result<Vec<u8>, SdkworkError>;

    async fn list_navigation(
        &self,
        publication_uuid: &str,
        locale: Option<&str>,
        cursor: Option<&str>,
        page_size: i64,
    ) -> Result<WikiPageListData, SdkworkError>;

    async fn search_pages(
        &self,
        publication_uuid: &str,
        query: &str,
        locale: Option<&str>,
        cursor: Option<&str>,
        page_size: i64,
    ) -> Result<WikiPageListData, SdkworkError>;
}

#[async_trait]
impl KnowledgebaseWikiSdkClient for KnowledgebaseInternalWikiApi {
    async fn retrieve_publication(
        &self,
        publication_uuid: &str,
    ) -> Result<WikiPublication, SdkworkError> {
        self.wiki_publications_retrieve(publication_uuid).await
    }

    async fn resolve_route(
        &self,
        publication_uuid: &str,
        request: &ResolveWikiRouteRequest,
    ) -> Result<WikiRouteResolution, SdkworkError> {
        self.wiki_publications_routes_resolve(publication_uuid, request)
            .await
    }

    async fn retrieve_content(
        &self,
        publication_uuid: &str,
        content_handle: &str,
    ) -> Result<Vec<u8>, SdkworkError> {
        self.wiki_publications_contents_retrieve(publication_uuid, content_handle)
            .await
    }

    async fn list_navigation(
        &self,
        publication_uuid: &str,
        locale: Option<&str>,
        cursor: Option<&str>,
        page_size: i64,
    ) -> Result<WikiPageListData, SdkworkError> {
        self.wiki_publications_navigation_list(publication_uuid, locale, cursor, Some(page_size))
            .await
    }

    async fn search_pages(
        &self,
        publication_uuid: &str,
        query: &str,
        locale: Option<&str>,
        cursor: Option<&str>,
        page_size: i64,
    ) -> Result<WikiPageListData, SdkworkError> {
        self.wiki_publications_pages_search(
            publication_uuid,
            query,
            locale,
            cursor,
            Some(page_size),
        )
        .await
    }
}

pub trait KnowledgebaseWikiSdkClientResolver: Send + Sync {
    fn resolve(
        &self,
        tenant_scope_hash: &str,
    ) -> WebsiteProviderResult<Arc<dyn KnowledgebaseWikiSdkClient>>;
}

pub struct FixedKnowledgebaseWikiSdkClientResolver {
    tenant_scope_hash: String,
    client: Arc<dyn KnowledgebaseWikiSdkClient>,
}

impl FixedKnowledgebaseWikiSdkClientResolver {
    pub fn new(
        tenant_scope_hash: impl Into<String>,
        client: Arc<dyn KnowledgebaseWikiSdkClient>,
    ) -> Result<Self, String> {
        let tenant_scope_hash = tenant_scope_hash.into();
        if tenant_scope_hash.is_empty()
            || tenant_scope_hash.len() > 256
            || tenant_scope_hash
                .bytes()
                .any(|byte| byte.is_ascii_control())
        {
            return Err(
                "tenant scope hash must be non-empty, bounded, and control-free".to_string(),
            );
        }
        Ok(Self {
            tenant_scope_hash,
            client,
        })
    }
}

impl KnowledgebaseWikiSdkClientResolver for FixedKnowledgebaseWikiSdkClientResolver {
    fn resolve(
        &self,
        tenant_scope_hash: &str,
    ) -> WebsiteProviderResult<Arc<dyn KnowledgebaseWikiSdkClient>> {
        if tenant_scope_hash != self.tenant_scope_hash {
            return Err(WebsiteProviderError::new(
                WebsiteProviderErrorKind::NotFound,
            ));
        }
        Ok(Arc::clone(&self.client))
    }
}
