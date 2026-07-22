use std::collections::BTreeSet;

use sdkwork_utils_rust::sha256_hash;
use sdkwork_webserver_core::website_runtime::WebsiteProviderType;
use serde_json::{Map, Value};
use thiserror::Error;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub const MAXIMUM_PROVIDER_EVENT_BYTES: usize = 256 * 1024;
const MAXIMUM_ROOT_SCOPES: usize = 256;

const DRIVE_VERSION_COMMITTED: &str = "drive.node.version.committed.v1";
const DRIVE_PATH_CHANGED: &str = "drive.node.path.changed.v1";
const DRIVE_ELIGIBILITY_CHANGED: &str = "drive.node.eligibility.changed.v1";
const DRIVE_DELETED: &str = "drive.node.deleted.v1";
const WIKI_PROVIDER_CHANGED: &str = "knowledgebase.wiki.provider.changed.v1";
const WIKI_ROUTE_CHANGED: &str = "knowledgebase.wiki.route.changed.v1";
const WIKI_ROUTE_REVOKED: &str = "knowledgebase.wiki.route.revoked.v1";
const WIKI_NAVIGATION_CHANGED: &str = "knowledgebase.wiki.navigation.changed.v1";
const WIKI_SEARCH_CHANGED: &str = "knowledgebase.wiki.search.changed.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebsiteProviderEventSource {
    Drive,
    Knowledgebase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebsiteProviderEventOrdering {
    Contiguous,
    Monotonic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebsiteProviderEventInvalidationPriority {
    Normal,
    Revocation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebsiteProviderEventInvalidationKind {
    Provider,
    Route { path: String },
    Navigation,
    Search,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebsiteProviderEventInvalidation {
    pub provider_type: WebsiteProviderType,
    pub provider_resource_uuid: String,
    pub kind: WebsiteProviderEventInvalidationKind,
    pub priority: WebsiteProviderEventInvalidationPriority,
    pub provider_generation: Option<String>,
    pub public_generation: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebsiteProviderEventScope {
    pub source: WebsiteProviderEventSource,
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub stream_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebsiteProviderEvent {
    pub id: String,
    pub event_type: String,
    pub sequence_no: u64,
    pub ordering: WebsiteProviderEventOrdering,
    pub scope: WebsiteProviderEventScope,
    pub invalidations: Vec<WebsiteProviderEventInvalidation>,
    pub payload_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum WebsiteProviderEventParseError {
    #[error("provider event body is empty or exceeds the bounded maximum")]
    InvalidSize,
    #[error("provider event body is not a supported strict JSON event")]
    InvalidContract,
    #[error("provider event contains a value outside its contract bounds")]
    InvalidValue,
    #[error("provider event type is unsupported")]
    UnsupportedType,
}

struct ParsedEnvelope<'a> {
    id: &'a str,
    event_type: &'a str,
    tenant_id: &'a str,
    organization_id: Option<&'a str>,
    sequence_no: u64,
    data: &'a Map<String, Value>,
}

pub fn parse_website_provider_event(
    body: &[u8],
) -> Result<WebsiteProviderEvent, WebsiteProviderEventParseError> {
    if body.is_empty() || body.len() > MAXIMUM_PROVIDER_EVENT_BYTES {
        return Err(WebsiteProviderEventParseError::InvalidSize);
    }
    let value: Value = serde_json::from_slice(body)
        .map_err(|_| WebsiteProviderEventParseError::InvalidContract)?;
    let object = value
        .as_object()
        .ok_or(WebsiteProviderEventParseError::InvalidContract)?;
    let event_type = required_string(object, "type")?;
    let payload_sha256 = sha256_hash(body);
    if event_type.starts_with("drive.") {
        parse_drive(object, payload_sha256)
    } else if event_type.starts_with("knowledgebase.wiki.") {
        parse_wiki(object, payload_sha256)
    } else {
        Err(WebsiteProviderEventParseError::UnsupportedType)
    }
}

fn parse_drive(
    object: &Map<String, Value>,
    payload_sha256: String,
) -> Result<WebsiteProviderEvent, WebsiteProviderEventParseError> {
    exact_keys(
        object,
        &[
            "id",
            "type",
            "source",
            "specversion",
            "time",
            "tenantId",
            "subject",
            "actorId",
            "sequenceNo",
            "data",
        ],
        &["organizationId"],
    )?;
    let envelope = parsed_envelope(object)?;
    if required_string(object, "source")? != "sdkwork-drive"
        || required_string(object, "specversion")? != "1.0"
    {
        return Err(WebsiteProviderEventParseError::InvalidContract);
    }
    validate_text(envelope.id, 64)?;
    validate_event_time(required_string(object, "time")?)?;
    validate_text(envelope.tenant_id, 64)?;
    validate_optional_text(envelope.organization_id, 64)?;
    validate_text(required_string(object, "actorId")?, 128)?;

    let (space_id, node_id, drive_uri, invalidations) = match envelope.event_type {
        DRIVE_VERSION_COMMITTED => parse_drive_version(envelope.data)?,
        DRIVE_PATH_CHANGED => parse_drive_path(envelope.data)?,
        DRIVE_ELIGIBILITY_CHANGED => parse_drive_eligibility(envelope.data)?,
        DRIVE_DELETED => parse_drive_deleted(envelope.data)?,
        _ => return Err(WebsiteProviderEventParseError::UnsupportedType),
    };
    validate_text(space_id, 64)?;
    validate_text(node_id, 64)?;
    let expected_subject = format!("drive://spaces/{space_id}/nodes/{node_id}");
    if drive_uri != expected_subject || required_string(object, "subject")? != expected_subject {
        return Err(WebsiteProviderEventParseError::InvalidValue);
    }
    let stream_id = format!(
        "drive:{}:{}:{space_id}",
        envelope.tenant_id,
        envelope.organization_id.unwrap_or("-")
    );
    Ok(WebsiteProviderEvent {
        id: envelope.id.to_owned(),
        event_type: envelope.event_type.to_owned(),
        sequence_no: envelope.sequence_no,
        ordering: WebsiteProviderEventOrdering::Contiguous,
        scope: WebsiteProviderEventScope {
            source: WebsiteProviderEventSource::Drive,
            tenant_id: envelope.tenant_id.to_owned(),
            organization_id: envelope.organization_id.map(str::to_owned),
            stream_id,
        },
        invalidations,
        payload_sha256,
    })
}

type DriveEventFields<'a> = (
    &'a str,
    &'a str,
    &'a str,
    Vec<WebsiteProviderEventInvalidation>,
);

fn parse_drive_version(
    data: &Map<String, Value>,
) -> Result<DriveEventFields<'_>, WebsiteProviderEventParseError> {
    exact_keys(
        data,
        &[
            "operationId",
            "spaceId",
            "nodeId",
            "driveUri",
            "driveVersionId",
            "versionNo",
            "spaceRelativePath",
            "contentType",
            "contentLength",
            "checksumSha256Hex",
            "rootScopes",
        ],
        &[],
    )?;
    validate_text(required_string(data, "operationId")?, 128)?;
    validate_text(required_string(data, "driveVersionId")?, 64)?;
    parse_positive(required_string(data, "versionNo")?)?;
    validate_path(required_string(data, "spaceRelativePath")?, 4096)?;
    validate_text(required_string(data, "contentType")?, 255)?;
    parse_non_negative(required_string(data, "contentLength")?)?;
    validate_sha256(required_string(data, "checksumSha256Hex")?)?;
    let invalidations = root_invalidations(
        required_array(data, "rootScopes")?,
        WebsiteProviderEventInvalidationPriority::Normal,
    )?;
    drive_identity(data, invalidations)
}

fn parse_drive_path(
    data: &Map<String, Value>,
) -> Result<DriveEventFields<'_>, WebsiteProviderEventParseError> {
    exact_keys(
        data,
        &[
            "operationId",
            "spaceId",
            "nodeId",
            "driveUri",
            "oldSpaceRelativePath",
            "newSpaceRelativePath",
            "oldRootScopes",
            "newRootScopes",
        ],
        &[],
    )?;
    validate_text(required_string(data, "operationId")?, 128)?;
    validate_path(required_string(data, "oldSpaceRelativePath")?, 4096)?;
    validate_path(required_string(data, "newSpaceRelativePath")?, 4096)?;
    let mut invalidations = root_invalidations(
        required_array(data, "oldRootScopes")?,
        WebsiteProviderEventInvalidationPriority::Revocation,
    )?;
    invalidations.extend(root_invalidations(
        required_array(data, "newRootScopes")?,
        WebsiteProviderEventInvalidationPriority::Normal,
    )?);
    drive_identity(data, invalidations)
}

fn parse_drive_eligibility(
    data: &Map<String, Value>,
) -> Result<DriveEventFields<'_>, WebsiteProviderEventParseError> {
    exact_keys(
        data,
        &[
            "operationId",
            "spaceId",
            "nodeId",
            "driveUri",
            "spaceRelativePath",
            "oldEligibility",
            "newEligibility",
            "reason",
            "rootScopes",
        ],
        &["driveVersionId", "versionNo"],
    )?;
    validate_text(required_string(data, "operationId")?, 128)?;
    validate_optional_text(optional_non_null_string(data, "driveVersionId")?, 64)?;
    if let Some(value) = optional_non_null_string(data, "versionNo")? {
        parse_positive(value)?;
    }
    validate_path(required_string(data, "spaceRelativePath")?, 4096)?;
    let old = required_string(data, "oldEligibility")?;
    let new = required_string(data, "newEligibility")?;
    if !matches!(old, "ELIGIBLE" | "INELIGIBLE") || !matches!(new, "ELIGIBLE" | "INELIGIBLE") {
        return Err(WebsiteProviderEventParseError::InvalidValue);
    }
    validate_upper_token(required_string(data, "reason")?)?;
    let priority = if new == "INELIGIBLE" {
        WebsiteProviderEventInvalidationPriority::Revocation
    } else {
        WebsiteProviderEventInvalidationPriority::Normal
    };
    let invalidations = root_invalidations(required_array(data, "rootScopes")?, priority)?;
    drive_identity(data, invalidations)
}

fn parse_drive_deleted(
    data: &Map<String, Value>,
) -> Result<DriveEventFields<'_>, WebsiteProviderEventParseError> {
    exact_keys(
        data,
        &[
            "operationId",
            "spaceId",
            "nodeId",
            "driveUri",
            "lastSpaceRelativePath",
            "deletionReason",
            "rootScopes",
        ],
        &["driveVersionId", "versionNo"],
    )?;
    validate_text(required_string(data, "operationId")?, 128)?;
    validate_optional_text(optional_non_null_string(data, "driveVersionId")?, 64)?;
    if let Some(value) = optional_non_null_string(data, "versionNo")? {
        parse_positive(value)?;
    }
    validate_path(required_string(data, "lastSpaceRelativePath")?, 4096)?;
    validate_upper_token(required_string(data, "deletionReason")?)?;
    let invalidations = root_invalidations(
        required_array(data, "rootScopes")?,
        WebsiteProviderEventInvalidationPriority::Revocation,
    )?;
    drive_identity(data, invalidations)
}

fn drive_identity<'a>(
    data: &'a Map<String, Value>,
    invalidations: Vec<WebsiteProviderEventInvalidation>,
) -> Result<DriveEventFields<'a>, WebsiteProviderEventParseError> {
    Ok((
        required_string(data, "spaceId")?,
        required_string(data, "nodeId")?,
        required_string(data, "driveUri")?,
        invalidations,
    ))
}

fn root_invalidations(
    roots: &[Value],
    priority: WebsiteProviderEventInvalidationPriority,
) -> Result<Vec<WebsiteProviderEventInvalidation>, WebsiteProviderEventParseError> {
    if roots.len() > MAXIMUM_ROOT_SCOPES {
        return Err(WebsiteProviderEventParseError::InvalidValue);
    }
    let mut invalidations = Vec::with_capacity(roots.len());
    for value in roots {
        let root = value
            .as_object()
            .ok_or(WebsiteProviderEventParseError::InvalidContract)?;
        exact_keys(
            root,
            &["scopeId", "scopeKind", "relativePath"],
            &["rootGeneration"],
        )?;
        let scope_id = required_string(root, "scopeId")?;
        let scope_kind = required_string(root, "scopeKind")?;
        let relative_path = required_string(root, "relativePath")?;
        validate_text(scope_id, 64)?;
        validate_path(relative_path, 4096)?;
        let root_generation = optional_non_null_string(root, "rootGeneration")?;
        if let Some(value) = root_generation {
            parse_positive(value)?;
        }
        if !matches!(scope_kind, "WEBSITE_ROOT" | "KNOWLEDGEBASE_RAW") {
            return Err(WebsiteProviderEventParseError::InvalidValue);
        }
        if scope_kind == "WEBSITE_ROOT" {
            invalidations.push(WebsiteProviderEventInvalidation {
                provider_type: WebsiteProviderType::Drive,
                provider_resource_uuid: scope_id.to_owned(),
                kind: WebsiteProviderEventInvalidationKind::Route {
                    path: relative_path.to_owned(),
                },
                priority,
                provider_generation: root_generation.map(str::to_owned),
                public_generation: None,
            });
        }
    }
    Ok(invalidations)
}

fn parse_wiki(
    object: &Map<String, Value>,
    payload_sha256: String,
) -> Result<WebsiteProviderEvent, WebsiteProviderEventParseError> {
    exact_keys(
        object,
        &[
            "id",
            "type",
            "source",
            "specversion",
            "time",
            "tenantId",
            "organizationId",
            "subject",
            "sequenceNo",
            "data",
        ],
        &[],
    )?;
    let envelope = parsed_envelope(object)?;
    if required_string(object, "source")? != "sdkwork-knowledgebase"
        || required_string(object, "specversion")? != "1.0"
        || !matches!(
            envelope.event_type,
            WIKI_PROVIDER_CHANGED
                | WIKI_ROUTE_CHANGED
                | WIKI_ROUTE_REVOKED
                | WIKI_NAVIGATION_CHANGED
                | WIKI_SEARCH_CHANGED
        )
    {
        return Err(WebsiteProviderEventParseError::InvalidContract);
    }
    validate_uuid(envelope.id)?;
    validate_event_time(required_string(object, "time")?)?;
    parse_positive(envelope.tenant_id)?;
    let organization_id = envelope
        .organization_id
        .ok_or(WebsiteProviderEventParseError::InvalidContract)?;
    parse_non_negative(organization_id)?;
    exact_keys(
        envelope.data,
        &[
            "providerResourceUuid",
            "providerGeneration",
            "navigationGeneration",
            "searchGeneration",
            "operation",
        ],
        &[
            "sourceFileUuid",
            "route",
            "pagePublicVersion",
            "previousPagePublicVersion",
            "driveCheckpoint",
            "reason",
        ],
    )?;
    let provider_resource_uuid = required_string(envelope.data, "providerResourceUuid")?;
    validate_uuid(provider_resource_uuid)?;
    if required_string(object, "subject")? != format!("wiki-publication:{provider_resource_uuid}") {
        return Err(WebsiteProviderEventParseError::InvalidValue);
    }
    let provider_generation = required_string(envelope.data, "providerGeneration")?;
    let navigation_generation = required_string(envelope.data, "navigationGeneration")?;
    let search_generation = required_string(envelope.data, "searchGeneration")?;
    parse_positive(provider_generation)?;
    parse_positive(navigation_generation)?;
    parse_positive(search_generation)?;
    if let Some(value) = nullable_optional_string(envelope.data, "sourceFileUuid")? {
        validate_uuid(value)?;
    }
    let route = nullable_optional_string(envelope.data, "route")?;
    if let Some(value) = route {
        validate_route(value)?;
    }
    let page_public_version = nullable_optional_string(envelope.data, "pagePublicVersion")?;
    if let Some(value) = page_public_version {
        parse_non_negative(value)?;
    }
    if let Some(value) = nullable_optional_string(envelope.data, "previousPagePublicVersion")? {
        parse_non_negative(value)?;
    }
    validate_upper_token(required_string(envelope.data, "operation")?)?;
    if let Some(value) = optional_non_null_string(envelope.data, "driveCheckpoint")? {
        parse_positive(value)?;
    }
    if let Some(value) = optional_non_null_string(envelope.data, "reason")? {
        validate_lower_token(value)?;
    }

    let kind = match envelope.event_type {
        WIKI_PROVIDER_CHANGED => WebsiteProviderEventInvalidationKind::Provider,
        WIKI_ROUTE_CHANGED | WIKI_ROUTE_REVOKED => WebsiteProviderEventInvalidationKind::Route {
            path: route
                .ok_or(WebsiteProviderEventParseError::InvalidValue)?
                .to_owned(),
        },
        WIKI_NAVIGATION_CHANGED => WebsiteProviderEventInvalidationKind::Navigation,
        WIKI_SEARCH_CHANGED => WebsiteProviderEventInvalidationKind::Search,
        _ => return Err(WebsiteProviderEventParseError::UnsupportedType),
    };
    let priority = if matches!(
        envelope.event_type,
        WIKI_PROVIDER_CHANGED | WIKI_ROUTE_REVOKED
    ) {
        WebsiteProviderEventInvalidationPriority::Revocation
    } else {
        WebsiteProviderEventInvalidationPriority::Normal
    };
    let public_generation = match kind {
        WebsiteProviderEventInvalidationKind::Navigation => Some(navigation_generation.to_owned()),
        WebsiteProviderEventInvalidationKind::Search => Some(search_generation.to_owned()),
        WebsiteProviderEventInvalidationKind::Route { .. } => {
            page_public_version.map(str::to_owned)
        }
        WebsiteProviderEventInvalidationKind::Provider => None,
    };
    let stream_id = format!(
        "knowledgebase:{}:{organization_id}:{provider_resource_uuid}",
        envelope.tenant_id
    );
    Ok(WebsiteProviderEvent {
        id: envelope.id.to_owned(),
        event_type: envelope.event_type.to_owned(),
        sequence_no: envelope.sequence_no,
        ordering: WebsiteProviderEventOrdering::Monotonic,
        scope: WebsiteProviderEventScope {
            source: WebsiteProviderEventSource::Knowledgebase,
            tenant_id: envelope.tenant_id.to_owned(),
            organization_id: Some(organization_id.to_owned()),
            stream_id,
        },
        invalidations: vec![WebsiteProviderEventInvalidation {
            provider_type: WebsiteProviderType::Knowledgebase,
            provider_resource_uuid: provider_resource_uuid.to_owned(),
            kind,
            priority,
            provider_generation: Some(provider_generation.to_owned()),
            public_generation,
        }],
        payload_sha256,
    })
}

fn parsed_envelope(
    object: &Map<String, Value>,
) -> Result<ParsedEnvelope<'_>, WebsiteProviderEventParseError> {
    Ok(ParsedEnvelope {
        id: required_string(object, "id")?,
        event_type: required_string(object, "type")?,
        tenant_id: required_string(object, "tenantId")?,
        organization_id: optional_non_null_string(object, "organizationId")?,
        sequence_no: parse_positive(required_string(object, "sequenceNo")?)?,
        data: object
            .get("data")
            .and_then(Value::as_object)
            .ok_or(WebsiteProviderEventParseError::InvalidContract)?,
    })
}

fn exact_keys(
    object: &Map<String, Value>,
    required: &[&str],
    optional: &[&str],
) -> Result<(), WebsiteProviderEventParseError> {
    let required = required.iter().copied().collect::<BTreeSet<_>>();
    let optional = optional.iter().copied().collect::<BTreeSet<_>>();
    if required.iter().any(|key| !object.contains_key(*key))
        || object
            .keys()
            .any(|key| !required.contains(key.as_str()) && !optional.contains(key.as_str()))
    {
        return Err(WebsiteProviderEventParseError::InvalidContract);
    }
    Ok(())
}

fn required_string<'a>(
    object: &'a Map<String, Value>,
    key: &str,
) -> Result<&'a str, WebsiteProviderEventParseError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or(WebsiteProviderEventParseError::InvalidContract)
}

fn optional_non_null_string<'a>(
    object: &'a Map<String, Value>,
    key: &str,
) -> Result<Option<&'a str>, WebsiteProviderEventParseError> {
    match object.get(key) {
        None => Ok(None),
        Some(value) => value
            .as_str()
            .map(Some)
            .ok_or(WebsiteProviderEventParseError::InvalidContract),
    }
}

fn nullable_optional_string<'a>(
    object: &'a Map<String, Value>,
    key: &str,
) -> Result<Option<&'a str>, WebsiteProviderEventParseError> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_str()
            .map(Some)
            .ok_or(WebsiteProviderEventParseError::InvalidContract),
    }
}

fn required_array<'a>(
    object: &'a Map<String, Value>,
    key: &str,
) -> Result<&'a [Value], WebsiteProviderEventParseError> {
    object
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or(WebsiteProviderEventParseError::InvalidContract)
}

fn validate_text(value: &str, maximum: usize) -> Result<(), WebsiteProviderEventParseError> {
    if value.is_empty()
        || value.len() > maximum
        || value.bytes().any(|byte| byte.is_ascii_control())
    {
        Err(WebsiteProviderEventParseError::InvalidValue)
    } else {
        Ok(())
    }
}

fn validate_optional_text(
    value: Option<&str>,
    maximum: usize,
) -> Result<(), WebsiteProviderEventParseError> {
    value.map_or(Ok(()), |value| validate_text(value, maximum))
}

fn validate_event_time(value: &str) -> Result<(), WebsiteProviderEventParseError> {
    validate_text(value, 64)?;
    OffsetDateTime::parse(value, &Rfc3339)
        .map(|_| ())
        .map_err(|_| WebsiteProviderEventParseError::InvalidValue)
}

fn validate_path(value: &str, maximum: usize) -> Result<(), WebsiteProviderEventParseError> {
    validate_text(value, maximum)
}

fn validate_route(value: &str) -> Result<(), WebsiteProviderEventParseError> {
    if !value.starts_with('/') {
        return Err(WebsiteProviderEventParseError::InvalidValue);
    }
    validate_text(value, 2048)
}

fn parse_positive(value: &str) -> Result<u64, WebsiteProviderEventParseError> {
    if value.is_empty()
        || value.starts_with('0')
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(WebsiteProviderEventParseError::InvalidValue);
    }
    value
        .parse::<u64>()
        .map_err(|_| WebsiteProviderEventParseError::InvalidValue)
}

fn parse_non_negative(value: &str) -> Result<u64, WebsiteProviderEventParseError> {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(WebsiteProviderEventParseError::InvalidValue);
    }
    value
        .parse::<u64>()
        .map_err(|_| WebsiteProviderEventParseError::InvalidValue)
}

fn validate_sha256(value: &str) -> Result<(), WebsiteProviderEventParseError> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(WebsiteProviderEventParseError::InvalidValue);
    };
    if hex.len() != 64 || !hex.bytes().all(is_lower_hex) {
        return Err(WebsiteProviderEventParseError::InvalidValue);
    }
    Ok(())
}

fn validate_uuid(value: &str) -> Result<(), WebsiteProviderEventParseError> {
    let bytes = value.as_bytes();
    if bytes.len() != 36
        || !bytes.iter().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                *byte == b'-'
            } else {
                is_lower_hex(*byte)
            }
        })
    {
        return Err(WebsiteProviderEventParseError::InvalidValue);
    }
    Ok(())
}

fn is_lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
}

fn validate_upper_token(value: &str) -> Result<(), WebsiteProviderEventParseError> {
    if value.is_empty()
        || value.len() > 64
        || !value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_uppercase() || (index > 0 && (byte.is_ascii_digit() || byte == b'_'))
        })
    {
        return Err(WebsiteProviderEventParseError::InvalidValue);
    }
    Ok(())
}

fn validate_lower_token(value: &str) -> Result<(), WebsiteProviderEventParseError> {
    if value.is_empty()
        || value.len() > 64
        || !value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_lowercase() || (index > 0 && (byte.is_ascii_digit() || byte == b'_'))
        })
    {
        return Err(WebsiteProviderEventParseError::InvalidValue);
    }
    Ok(())
}
