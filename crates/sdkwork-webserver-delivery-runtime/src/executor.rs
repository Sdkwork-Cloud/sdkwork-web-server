use std::{future::Future, sync::Arc, time::Duration};

use sdkwork_webserver_contract::provider::{
    OpenWebsiteContentRequest, OpenedWebsiteContent, ResolveWebsiteStaticPathRequest,
    ResolveWebsiteWikiRouteRequest, ResolvedWebsiteContent, WebsiteByteRange,
    WebsiteContentMetadata, WebsiteContentRange, WebsiteContentResolution, WebsiteProviderError,
    WebsiteProviderErrorKind, WebsiteProviderPurpose, WebsiteProviderResult,
    WebsiteProviderRuntimeContext, WebsiteStaticContentProvider, WebsiteWikiProvider,
    WebsiteWikiRouteResolution,
};
use sdkwork_webserver_core::website_runtime::{
    SelectedWebsiteRoute, WebsiteHandler, WebsiteMountMode, WebsiteRequestRoutingContext,
    WebsiteRouteSelection, WebsiteRuntimeRegistry,
};
use tokio::time::{timeout, Instant};

use crate::{
    stream::BoundedProviderContentStream, WebsiteDeliveryContent, WebsiteDeliveryContentKind,
    WebsiteDeliveryError, WebsiteDeliveryMethod, WebsiteDeliveryOutcome, WebsiteDeliveryRedirect,
    WebsiteDeliveryRequest, WebsiteDeliveryRouteIdentity, WebsiteDeliveryScheme,
    WebsiteProviderRegistry,
};

const MAXIMUM_REQUEST_ID_BYTES: usize = 256;
const MAXIMUM_TRACE_ID_BYTES: usize = 256;

pub struct WebsiteDeliveryExecutor {
    runtime_registry: Arc<WebsiteRuntimeRegistry>,
    provider_registry: Arc<WebsiteProviderRegistry>,
}

impl WebsiteDeliveryExecutor {
    pub fn new(
        runtime_registry: Arc<WebsiteRuntimeRegistry>,
        provider_registry: Arc<WebsiteProviderRegistry>,
    ) -> Self {
        Self {
            runtime_registry,
            provider_registry,
        }
    }

    pub async fn execute(
        &self,
        request: WebsiteDeliveryRequest,
    ) -> Result<WebsiteDeliveryOutcome, WebsiteDeliveryError> {
        validate_request_identity(&request)?;
        let runtime_set = self
            .runtime_registry
            .current()
            .ok_or(WebsiteDeliveryError::RuntimeUnavailable)?;
        let routing_context = WebsiteRequestRoutingContext {
            verified_preferred_variant_uuid: request
                .routing
                .verified_preferred_variant_uuid
                .as_deref(),
            client_class: request.routing.client_class,
            client_classification_source: request.routing.client_classification_source,
        };
        let Some(selection) =
            runtime_set.select_route(&request.authority, &request.path, routing_context)?
        else {
            return Ok(WebsiteDeliveryOutcome::NotFound);
        };
        let runtime_set_generation = runtime_set.generation();

        match selection {
            WebsiteRouteSelection::Redirect(redirect) => Ok(WebsiteDeliveryOutcome::Redirect(
                WebsiteDeliveryRedirect::Binding {
                    status_code: redirect.status_code,
                    scheme: redirect.scheme,
                    hostname: redirect.hostname.to_owned(),
                    path: redirect.path,
                    preserve_query: redirect.preserve_query,
                },
            )),
            WebsiteRouteSelection::Serve(selected) => {
                let route = OwnedSelectedRoute::from_selected(runtime_set_generation, selected);
                if route.force_https && request.scheme == WebsiteDeliveryScheme::Http {
                    return Ok(WebsiteDeliveryOutcome::Redirect(
                        WebsiteDeliveryRedirect::Binding {
                            status_code: 308,
                            scheme: sdkwork_webserver_core::website_runtime::WebsiteRedirectScheme::Https,
                            hostname: route.normalized_request_hostname,
                            path: route.normalized_request_path,
                            preserve_query: true,
                        },
                    ));
                }
                match route.handler {
                    WebsiteHandler::Wiki => self.execute_wiki(route, request).await,
                    WebsiteHandler::Static | WebsiteHandler::Spa => {
                        self.execute_static(route, request).await
                    }
                }
            }
        }
    }

    async fn execute_wiki(
        &self,
        route: OwnedSelectedRoute,
        request: WebsiteDeliveryRequest,
    ) -> Result<WebsiteDeliveryOutcome, WebsiteDeliveryError> {
        let provider = self
            .provider_registry
            .wiki_provider(route.identity.provider.provider_type)
            .ok_or(WebsiteDeliveryError::ProviderNotRegistered {
                provider_type: route.identity.provider.provider_type,
                capability: "wiki",
            })?;
        let deadline = ProviderDeadline::new(route.provider_timeout_ms);
        let mut context = route.provider_context(&request);
        context.deadline_ms = deadline.remaining_ms()?;
        let resolve_request = ResolveWebsiteWikiRouteRequest {
            context: context.clone(),
            provider: route.identity.provider.clone(),
            route: route.identity.provider_relative_path.clone(),
            locale: request.locale.clone(),
            conditions: request.conditions.clone(),
        };
        let resolution = match deadline
            .call(provider.resolve_wiki_route(&resolve_request))
            .await
        {
            Ok(resolution) => resolution,
            Err(error) => return provider_error_outcome(error),
        };

        match resolution {
            WebsiteWikiRouteResolution::NotModified => Ok(WebsiteDeliveryOutcome::NotModified),
            WebsiteWikiRouteResolution::Redirect(redirect) => {
                let canonical_route = route.public_route(&redirect.canonical_route)?;
                Ok(WebsiteDeliveryOutcome::Redirect(
                    WebsiteDeliveryRedirect::Wiki {
                        route: Box::new(route.identity),
                        status_code: redirect.status_code,
                        canonical_route,
                        preserve_query: true,
                    },
                ))
            }
            WebsiteWikiRouteResolution::Content(content) => {
                enforce_content_policy(
                    &content.metadata,
                    route.maximum_object_bytes,
                    request.range,
                )?;
                let opened = Self::open_wiki_body(
                    provider,
                    &route,
                    &request,
                    context,
                    &deadline,
                    content.content_handle.clone(),
                    content.metadata.content_length,
                )
                .await?;
                let canonical_route = route.public_route(&content.canonical_route)?;
                let opened = opened_body_fields(
                    opened,
                    &content.metadata,
                    request.range,
                    request.conditions.if_range.is_some(),
                    route.maximum_object_bytes,
                    route.provider_timeout_ms,
                )?;
                Ok(WebsiteDeliveryOutcome::Content(Box::new(
                    WebsiteDeliveryContent {
                        route: route.identity,
                        kind: WebsiteDeliveryContentKind::Wiki(content.kind),
                        metadata: content.metadata,
                        response_content_length: opened.content_length,
                        content_range: opened.content_range,
                        canonical_route: Some(canonical_route),
                        page_uuid: content.page_uuid,
                        public_page_version: Some(content.public_page_version),
                        renderer_version: Some(content.renderer_version),
                        navigation_generation: Some(content.navigation_generation),
                        search_generation: Some(content.search_generation),
                        body: opened.stream,
                    },
                )))
            }
        }
    }

    async fn open_wiki_body(
        provider: Arc<dyn WebsiteWikiProvider>,
        route: &OwnedSelectedRoute,
        request: &WebsiteDeliveryRequest,
        mut context: WebsiteProviderRuntimeContext,
        deadline: &ProviderDeadline,
        content_handle: sdkwork_webserver_contract::provider::WebsiteProviderContentHandle,
        expected_bytes: u64,
    ) -> Result<Option<OpenedWebsiteContent>, WebsiteDeliveryError> {
        if request.method == WebsiteDeliveryMethod::Head {
            return Ok(None);
        }
        context.deadline_ms = deadline.remaining_ms()?;
        let open_request = OpenWebsiteContentRequest {
            context,
            provider: route.identity.provider.clone(),
            provider_relative_path: route.identity.provider_relative_path.clone(),
            content_handle,
            range: request.range,
            conditions: request.conditions.clone(),
            maximum_bytes: route.maximum_object_bytes,
        };
        let opened = deadline
            .call(provider.open_wiki_content(&open_request))
            .await?;
        if request.range.is_none() && opened.content_length != expected_bytes {
            return Err(provider_contract_mismatch());
        }
        Ok(Some(opened))
    }

    async fn execute_static(
        &self,
        route: OwnedSelectedRoute,
        request: WebsiteDeliveryRequest,
    ) -> Result<WebsiteDeliveryOutcome, WebsiteDeliveryError> {
        let provider = self
            .provider_registry
            .static_provider(route.identity.provider.provider_type)
            .ok_or(WebsiteDeliveryError::ProviderNotRegistered {
                provider_type: route.identity.provider.provider_type,
                capability: "static-content",
            })?;
        let deadline = ProviderDeadline::new(route.provider_timeout_ms);
        let mut context = route.provider_context(&request);
        for candidate in static_candidates(&route, request.spa_fallback_eligible) {
            context.deadline_ms = deadline.remaining_ms()?;
            let resolve_request = ResolveWebsiteStaticPathRequest {
                context: context.clone(),
                provider: route.identity.provider.clone(),
                provider_relative_path: candidate.clone(),
                conditions: request.conditions.clone(),
            };
            let resolution = deadline
                .call(provider.resolve_static_path(&resolve_request))
                .await;
            let content = match resolution {
                Ok(WebsiteContentResolution::Found(content)) => content,
                Ok(WebsiteContentResolution::NotModified) => {
                    return Ok(WebsiteDeliveryOutcome::NotModified)
                }
                Err(error) if provider_error_is_not_found(&error) => continue,
                Err(error) if error.kind == WebsiteProviderErrorKind::NotModified => {
                    return Ok(WebsiteDeliveryOutcome::NotModified)
                }
                Err(error) => return Err(error.into()),
            };
            enforce_content_policy(&content.metadata, route.maximum_object_bytes, request.range)?;
            return Self::open_static_content(
                provider, route, request, context, &deadline, candidate, content,
            )
            .await;
        }
        Ok(WebsiteDeliveryOutcome::NotFound)
    }

    async fn open_static_content(
        provider: Arc<dyn WebsiteStaticContentProvider>,
        mut route: OwnedSelectedRoute,
        request: WebsiteDeliveryRequest,
        mut context: WebsiteProviderRuntimeContext,
        deadline: &ProviderDeadline,
        candidate: String,
        content: ResolvedWebsiteContent,
    ) -> Result<WebsiteDeliveryOutcome, WebsiteDeliveryError> {
        route.identity.provider_relative_path = candidate;
        let expected_bytes = content.metadata.content_length;
        let if_range_present = request.conditions.if_range.is_some();
        let opened = if request.method == WebsiteDeliveryMethod::Head {
            None
        } else {
            context.deadline_ms = deadline.remaining_ms()?;
            let open_request = OpenWebsiteContentRequest {
                context,
                provider: route.identity.provider.clone(),
                provider_relative_path: route.identity.provider_relative_path.clone(),
                content_handle: content.content_handle,
                range: request.range,
                conditions: request.conditions,
                maximum_bytes: route.maximum_object_bytes,
            };
            let opened = deadline
                .call(provider.open_static_content(&open_request))
                .await?;
            if request.range.is_none() && opened.content_length != expected_bytes {
                return Err(provider_contract_mismatch());
            }
            Some(opened)
        };
        let opened = opened_body_fields(
            opened,
            &content.metadata,
            request.range,
            if_range_present,
            route.maximum_object_bytes,
            route.provider_timeout_ms,
        )?;
        Ok(WebsiteDeliveryOutcome::Content(Box::new(
            WebsiteDeliveryContent {
                route: route.identity,
                kind: WebsiteDeliveryContentKind::Static,
                metadata: content.metadata,
                response_content_length: opened.content_length,
                content_range: opened.content_range,
                canonical_route: None,
                page_uuid: None,
                public_page_version: None,
                renderer_version: None,
                navigation_generation: None,
                search_generation: None,
                body: opened.stream,
            },
        )))
    }
}

struct OwnedSelectedRoute {
    identity: WebsiteDeliveryRouteIdentity,
    handler: WebsiteHandler,
    index_files: Vec<String>,
    spa_fallback: Option<String>,
    directory_request: bool,
    binding_path_prefix: String,
    mount_path_prefix: String,
    mount_mode: WebsiteMountMode,
    resource_subpath: String,
    normalized_request_hostname: String,
    normalized_request_path: String,
    force_https: bool,
    provider_timeout_ms: u64,
    maximum_object_bytes: u64,
}

struct OpenedDeliveryBody {
    stream: Option<Box<dyn sdkwork_webserver_contract::provider::WebsiteProviderContentStream>>,
    content_length: u64,
    content_range: Option<WebsiteContentRange>,
}

impl OwnedSelectedRoute {
    fn from_selected(runtime_set_generation: u64, selected: SelectedWebsiteRoute<'_>) -> Self {
        Self {
            identity: WebsiteDeliveryRouteIdentity {
                runtime_set_generation,
                revision_uuid: selected.revision_uuid.to_owned(),
                tenant_scope_hash: selected.tenant_scope_hash.to_owned(),
                site_uuid: selected.site_uuid.to_owned(),
                binding_uuid: selected.binding.binding_uuid.clone(),
                variant_uuid: selected.variant.variant_uuid.clone(),
                mount_uuid: selected.mount.mount_uuid.clone(),
                resource_uuid: selected.resource.resource_uuid.clone(),
                provider: selected.provider.clone(),
                provider_relative_path: selected.provider_relative_path,
                variant_reason: selected.variant_reason,
            },
            handler: selected.mount.handler,
            index_files: selected.mount.index_files.clone(),
            spa_fallback: selected.mount.spa_fallback.clone(),
            directory_request: selected.normalized_request_path.ends_with('/'),
            binding_path_prefix: selected.binding.path_prefix.clone(),
            mount_path_prefix: selected.mount.path_prefix.clone(),
            mount_mode: selected.mount.translation.mode,
            resource_subpath: selected.mount.translation.resource_subpath.clone(),
            normalized_request_hostname: selected.normalized_request_hostname,
            normalized_request_path: selected.normalized_request_path,
            force_https: selected.force_https,
            provider_timeout_ms: selected.provider_timeout_ms,
            maximum_object_bytes: selected.maximum_object_bytes,
        }
    }

    fn provider_context(&self, request: &WebsiteDeliveryRequest) -> WebsiteProviderRuntimeContext {
        WebsiteProviderRuntimeContext {
            tenant_scope_hash: self.identity.tenant_scope_hash.clone(),
            site_uuid: self.identity.site_uuid.clone(),
            binding_uuid: self.identity.binding_uuid.clone(),
            variant_uuid: self.identity.variant_uuid.clone(),
            mount_uuid: self.identity.mount_uuid.clone(),
            resource_uuid: self.identity.resource_uuid.clone(),
            request_id: request.request_id.clone(),
            trace_id: request.trace_id.clone(),
            deadline_ms: self.provider_timeout_ms,
            purpose: WebsiteProviderPurpose::Request,
        }
    }

    fn public_route(&self, provider_route: &str) -> Result<String, WebsiteDeliveryError> {
        let translated = strip_segment_prefix(provider_route, &self.resource_subpath)
            .ok_or_else(provider_contract_mismatch)?;
        let binding_relative = match self.mount_mode {
            WebsiteMountMode::Root => {
                if !segment_prefix_matches(&self.mount_path_prefix, &translated) {
                    return Err(provider_contract_mismatch());
                }
                translated
            }
            WebsiteMountMode::Alias => join_canonical_paths(&self.mount_path_prefix, &translated),
        };
        Ok(join_canonical_paths(
            &self.binding_path_prefix,
            &binding_relative,
        ))
    }
}

fn static_candidates(route: &OwnedSelectedRoute, spa_fallback_eligible: bool) -> Vec<String> {
    let mut candidates = if route.directory_request {
        route
            .index_files
            .iter()
            .map(|index| join_provider_path(&route.identity.provider_relative_path, index))
            .collect::<Vec<_>>()
    } else {
        vec![route.identity.provider_relative_path.clone()]
    };
    if route.handler == WebsiteHandler::Spa && spa_fallback_eligible {
        if let Some(fallback) = &route.spa_fallback {
            if !candidates.contains(fallback) {
                candidates.push(fallback.clone());
            }
        }
    }
    candidates
}

fn join_provider_path(directory: &str, index: &str) -> String {
    if directory == "/" {
        format!("/{index}")
    } else {
        format!(
            "{}{index}",
            directory.trim_end_matches('/').to_owned() + "/"
        )
    }
}

fn segment_prefix_matches(prefix: &str, path: &str) -> bool {
    prefix == "/"
        || path == prefix
        || path
            .strip_prefix(prefix)
            .is_some_and(|remainder| remainder.starts_with('/'))
}

fn strip_segment_prefix(path: &str, prefix: &str) -> Option<String> {
    if prefix == "/" {
        return Some(path.to_owned());
    }
    if path == prefix {
        return Some("/".to_owned());
    }
    path.strip_prefix(prefix)
        .filter(|remainder| remainder.starts_with('/'))
        .map(str::to_owned)
}

fn join_canonical_paths(prefix: &str, suffix: &str) -> String {
    match (prefix, suffix) {
        ("/", suffix) => suffix.to_owned(),
        (prefix, "/") => prefix.to_owned(),
        (prefix, suffix) => format!("{prefix}{suffix}"),
    }
}

fn enforce_content_policy(
    metadata: &WebsiteContentMetadata,
    maximum_bytes: u64,
    range: Option<sdkwork_webserver_contract::provider::WebsiteByteRange>,
) -> Result<(), WebsiteDeliveryError> {
    if metadata.content_length > maximum_bytes {
        return Err(WebsiteDeliveryError::ContentTooLarge {
            declared_bytes: metadata.content_length,
            maximum_bytes,
        });
    }
    if range.is_some() && !metadata.range_supported {
        return Err(WebsiteDeliveryError::RangeNotSupported);
    }
    Ok(())
}

fn opened_body_fields(
    opened: Option<OpenedWebsiteContent>,
    metadata: &WebsiteContentMetadata,
    requested_range: Option<WebsiteByteRange>,
    if_range_present: bool,
    maximum_bytes: u64,
    chunk_timeout_ms: u64,
) -> Result<OpenedDeliveryBody, WebsiteDeliveryError> {
    let Some(opened) = opened else {
        return Ok(OpenedDeliveryBody {
            stream: None,
            content_length: metadata.content_length,
            content_range: None,
        });
    };
    validate_opened_content(
        &opened,
        metadata,
        requested_range,
        if_range_present,
        maximum_bytes,
    )?;
    let response_content_length = opened.content_length;
    let content_range = opened.content_range;
    let body = Box::new(BoundedProviderContentStream::new(
        opened.stream,
        maximum_bytes,
        Some(response_content_length),
        chunk_timeout_ms,
    ));
    Ok(OpenedDeliveryBody {
        stream: Some(body),
        content_length: response_content_length,
        content_range,
    })
}

fn validate_opened_content(
    opened: &OpenedWebsiteContent,
    metadata: &WebsiteContentMetadata,
    requested_range: Option<WebsiteByteRange>,
    if_range_present: bool,
    maximum_bytes: u64,
) -> Result<(), WebsiteDeliveryError> {
    if opened.content_length > maximum_bytes {
        return Err(WebsiteDeliveryError::ContentTooLarge {
            declared_bytes: opened.content_length,
            maximum_bytes,
        });
    }
    match (requested_range, opened.content_range) {
        (None, None) if opened.content_length == metadata.content_length => Ok(()),
        (Some(_), None) if if_range_present && opened.content_length == metadata.content_length => {
            Ok(())
        }
        (Some(requested), Some(actual)) => {
            let requested_end = requested
                .end_inclusive
                .unwrap_or_else(|| metadata.content_length.saturating_sub(1));
            if metadata.content_length == 0
                || requested.start >= metadata.content_length
                || requested_end < requested.start
                || actual.start != requested.start
                || actual.end_inclusive != requested_end.min(metadata.content_length - 1)
                || actual.complete_length != metadata.content_length
                || opened.content_length
                    != actual
                        .end_inclusive
                        .checked_sub(actual.start)
                        .and_then(|length| length.checked_add(1))
                        .ok_or_else(provider_contract_mismatch)?
            {
                return Err(provider_contract_mismatch());
            }
            Ok(())
        }
        _ => Err(provider_contract_mismatch()),
    }
}

fn validate_request_identity(request: &WebsiteDeliveryRequest) -> Result<(), WebsiteDeliveryError> {
    if !valid_bounded_identity(&request.request_id, MAXIMUM_REQUEST_ID_BYTES)
        || !valid_bounded_identity(&request.trace_id, MAXIMUM_TRACE_ID_BYTES)
    {
        return Err(WebsiteDeliveryError::InvalidRequestIdentity);
    }
    Ok(())
}

fn valid_bounded_identity(value: &str, maximum_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum_bytes
        && !value.bytes().any(|byte| byte.is_ascii_control())
}

fn provider_error_is_not_found(error: &WebsiteProviderError) -> bool {
    matches!(
        error.kind,
        WebsiteProviderErrorKind::NotFound
            | WebsiteProviderErrorKind::NotPublic
            | WebsiteProviderErrorKind::Revoked
    )
}

fn provider_error_outcome(
    error: WebsiteProviderError,
) -> Result<WebsiteDeliveryOutcome, WebsiteDeliveryError> {
    if provider_error_is_not_found(&error) {
        Ok(WebsiteDeliveryOutcome::NotFound)
    } else if error.kind == WebsiteProviderErrorKind::NotModified {
        Ok(WebsiteDeliveryOutcome::NotModified)
    } else {
        Err(error.into())
    }
}

struct ProviderDeadline {
    expires_at: Instant,
}

impl ProviderDeadline {
    fn new(timeout_ms: u64) -> Self {
        Self {
            expires_at: Instant::now() + Duration::from_millis(timeout_ms),
        }
    }

    fn remaining_ms(&self) -> WebsiteProviderResult<u64> {
        let remaining = self.expires_at.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(provider_deadline_exceeded());
        }
        Ok(u64::try_from(remaining.as_millis())
            .unwrap_or(u64::MAX)
            .max(1))
    }

    async fn call<T, F>(&self, future: F) -> WebsiteProviderResult<T>
    where
        F: Future<Output = WebsiteProviderResult<T>>,
    {
        let remaining_ms = self.remaining_ms()?;
        timeout(Duration::from_millis(remaining_ms), future)
            .await
            .map_err(|_| provider_deadline_exceeded())?
    }
}

fn provider_deadline_exceeded() -> WebsiteProviderError {
    WebsiteProviderError::new(WebsiteProviderErrorKind::DeadlineExceeded)
}

fn provider_contract_mismatch() -> WebsiteDeliveryError {
    WebsiteProviderError::new(WebsiteProviderErrorKind::ContractMismatch).into()
}
