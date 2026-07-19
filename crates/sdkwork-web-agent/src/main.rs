//! SDKWork Web Node Daemon using the legacy v3 Agent API compatibility contract.

#[path = "state.rs"]
mod state;

use std::collections::HashSet;
use std::time::Duration;

use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkResourceData, SDKWORK_SUCCESS_CODE};
use sdkwork_webserver_contract::{
    AgentCertificateBundle, AgentHeartbeatRequest, AgentHeartbeatResponse, AgentSyncResponse,
};
use sdkwork_webserver_edge_runtime::EdgeRuntime;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use state::{resolve_state_path, NodeDaemonLock, NodeDaemonState};
use tracing::{info, warn};

const NODE_DAEMON_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_SYNC_INTERVAL_SECS: u64 = 30;
const MIN_SYNC_INTERVAL_SECS: u64 = 1;
const MAX_SYNC_INTERVAL_SECS: u64 = 3_600;
const HTTP_TIMEOUT_SECS: u64 = 60;
const MAX_HEARTBEAT_RESPONSE_BYTES: usize = 64 * 1024;
const MAX_SYNC_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
const MAX_NGINX_CONFIGS_PER_SYNC: usize = 2_048;
const MAX_CERTIFICATES_PER_SYNC: usize = 2_048;

struct NodeDaemonRuntimeConfig {
    control_plane: reqwest::Url,
    node_token: String,
    interval_secs: u64,
}

impl NodeDaemonRuntimeConfig {
    fn from_env() -> anyhow::Result<Self> {
        let control_plane = parse_control_plane_url(
            &std::env::var("SDKWORK_WEB_CONTROL_PLANE_URL")
                .map_err(|_| anyhow::anyhow!("SDKWORK_WEB_CONTROL_PLANE_URL is required"))?,
        )?;
        let node_token = required_env_alias("SDKWORK_WEB_NODE_TOKEN", "SDKWORK_WEB_AGENT_TOKEN")?;
        validate_node_token(&node_token)?;
        let interval_secs = read_env_alias(
            "SDKWORK_WEB_NODE_SYNC_INTERVAL_SECS",
            "SDKWORK_WEB_AGENT_SYNC_INTERVAL_SECS",
        )?
        .map(|value| parse_sync_interval(&value))
        .transpose()?
        .unwrap_or(DEFAULT_SYNC_INTERVAL_SECS);
        Ok(Self {
            control_plane,
            node_token,
            interval_secs,
        })
    }
}

pub async fn run() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let edge = EdgeRuntime::from_env()?;
    let runtime = NodeDaemonRuntimeConfig::from_env()?;
    let state_path = resolve_state_path()?;
    let _node_daemon_lock = NodeDaemonLock::acquire(&state_path)?;
    let mut local_state = NodeDaemonState::load(&state_path)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()?;

    info!(
        interval_secs = runtime.interval_secs,
        nginx_enabled = edge.config().nginx_enabled,
        state_revision = local_state.revision(),
        state_pending = local_state.is_pending(),
        desired_sync_version = local_state.desired_sync_version(),
        observed_sync_version = local_state.observed_sync_version(),
        "sdkwork web node daemon started"
    );

    loop {
        if let Err(error) = sync_once(&edge, &runtime, &client, &state_path, &mut local_state).await
        {
            warn!(error = %error, "node sync cycle failed");
        }
        tokio::time::sleep(Duration::from_secs(runtime.interval_secs)).await;
    }
}

async fn sync_once(
    edge: &EdgeRuntime,
    runtime: &NodeDaemonRuntimeConfig,
    client: &reqwest::Client,
    state_path: &std::path::Path,
    local_state: &mut NodeDaemonState,
) -> anyhow::Result<()> {
    let heartbeat_url = runtime
        .control_plane
        .join("/backend/v3/api/agent/heartbeat")?;
    let heartbeat = AgentHeartbeatRequest {
        agent_version: Some(NODE_DAEMON_VERSION.to_string()),
        nginx_enabled: Some(edge.config().nginx_enabled),
        active_configs: None,
        last_sync_version: local_state.observed_sync_version().map(str::to_string),
    };
    let heartbeat_response = client
        .post(heartbeat_url)
        .header("X-SDKWork-Agent-Token", &runtime.node_token)
        .json(&heartbeat)
        .send()
        .await?
        .error_for_status()?;
    let heartbeat_ack: AgentHeartbeatResponse = decode_resource_response(
        &read_body_bounded(heartbeat_response, MAX_HEARTBEAT_RESPONSE_BYTES).await?,
    )?;
    if heartbeat_ack.server_id.trim().is_empty() {
        anyhow::bail!("control-plane heartbeat acknowledgement has an empty serverId");
    }
    if heartbeat_ack.status != 1 {
        anyhow::bail!("control-plane heartbeat acknowledgement did not mark the node active");
    }

    let mut sync_url = runtime.control_plane.join("/backend/v3/api/agent/sync")?;
    if let Some(last_sync_version) = local_state.observed_sync_version() {
        sync_url
            .query_pairs_mut()
            .append_pair("ifSyncVersion", last_sync_version);
    }

    let response = client
        .get(sync_url)
        .header("X-SDKWork-Agent-Token", &runtime.node_token)
        .send()
        .await?
        .error_for_status()?;
    let manifest: AgentSyncResponse =
        decode_resource_response(&read_body_bounded(response, MAX_SYNC_RESPONSE_BYTES).await?)?;
    validate_manifest_bounds(&manifest)?;
    if manifest.server_id != heartbeat_ack.server_id {
        anyhow::bail!("node identity mismatch between heartbeat acknowledgement and sync manifest");
    }

    if manifest.unchanged {
        if local_state.observed_sync_version() != Some(manifest.sync_version.as_str()) {
            anyhow::bail!(
                "control plane reported unchanged for a version the Web Node Daemon has not observed"
            );
        }
        info!(
            server_id = %manifest.server_id,
            sync_version = %manifest.sync_version,
            "node sync manifest unchanged"
        );
        return Ok(());
    }

    let desired_state = local_state.with_desired(&manifest.sync_version)?;
    desired_state.save(state_path)?;
    *local_state = desired_state;

    info!(
        server_id = %manifest.server_id,
        sync_version = %manifest.sync_version,
        nginx_configs = manifest.nginx_configs.len(),
        certificates = manifest.certificates.len(),
        "node sync manifest received"
    );

    for config in &manifest.nginx_configs {
        edge.deploy_site_config(&config.domain, &config.config_content)
            .map_err(|error| anyhow::anyhow!("deploy nginx site {}: {error}", config.domain))?;
    }

    for certificate in &manifest.certificates {
        apply_certificate_bundle(edge, certificate)?;
    }

    edge.reload()?;

    let observed_state = local_state.with_observed(&manifest.sync_version)?;
    observed_state.save(state_path)?;
    *local_state = observed_state;

    Ok(())
}

async fn read_body_bounded(
    mut response: reqwest::Response,
    maximum_bytes: usize,
) -> anyhow::Result<Vec<u8>> {
    if response
        .content_length()
        .is_some_and(|length| length > maximum_bytes as u64)
    {
        anyhow::bail!("control-plane response exceeds {maximum_bytes} bytes");
    }
    let capacity = response
        .content_length()
        .and_then(|length| usize::try_from(length).ok())
        .unwrap_or(0)
        .min(maximum_bytes);
    let mut body = Vec::with_capacity(capacity);
    while let Some(chunk) = response.chunk().await? {
        let next_length = body
            .len()
            .checked_add(chunk.len())
            .ok_or_else(|| anyhow::anyhow!("control-plane response length overflow"))?;
        if next_length > maximum_bytes {
            anyhow::bail!("control-plane response exceeds {maximum_bytes} bytes");
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn decode_resource_response<T>(body: &[u8]) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let response: SdkWorkApiResponse<SdkWorkResourceData<T>> = serde_json::from_slice(body)
        .map_err(|error| anyhow::anyhow!("invalid SDKWork resource response: {error}"))?;
    if response.code != SDKWORK_SUCCESS_CODE {
        anyhow::bail!(
            "control-plane resource response returned business code {} (traceId={})",
            response.code,
            response.trace_id
        );
    }
    if response.trace_id.trim().is_empty() {
        anyhow::bail!("control-plane resource response has an empty traceId");
    }
    Ok(response.data.item)
}

fn validate_manifest_bounds(manifest: &AgentSyncResponse) -> anyhow::Result<()> {
    if manifest.nginx_configs.len() > MAX_NGINX_CONFIGS_PER_SYNC {
        anyhow::bail!(
            "node sync contains more than {MAX_NGINX_CONFIGS_PER_SYNC} Nginx configurations"
        );
    }
    if manifest.certificates.len() > MAX_CERTIFICATES_PER_SYNC {
        anyhow::bail!("node sync contains more than {MAX_CERTIFICATES_PER_SYNC} certificates");
    }
    if manifest.unchanged
        && (!manifest.nginx_configs.is_empty() || !manifest.certificates.is_empty())
    {
        anyhow::bail!("unchanged node sync response must not contain deployment bundles");
    }
    let mut config_ids = HashSet::with_capacity(manifest.nginx_configs.len());
    let mut config_domains = HashSet::with_capacity(manifest.nginx_configs.len());
    for config in &manifest.nginx_configs {
        if config.version < 0 {
            anyhow::bail!("node sync contains a negative Nginx configuration version");
        }
        if !config_ids.insert(config.config_id.as_str()) {
            anyhow::bail!("node sync contains a duplicate Nginx configuration ID");
        }
        if !config_domains.insert(config.domain.to_ascii_lowercase()) {
            anyhow::bail!("node sync contains a duplicate Nginx activation domain");
        }
        let fingerprint = hex::encode(Sha256::digest(config.config_content.as_bytes()));
        if config.fingerprint != fingerprint {
            anyhow::bail!("node sync Nginx configuration fingerprint mismatch");
        }
    }
    let mut certificate_ids = HashSet::with_capacity(manifest.certificates.len());
    let mut certificate_names = HashSet::with_capacity(manifest.certificates.len());
    for certificate in &manifest.certificates {
        if !certificate_ids.insert(certificate.certificate_id.as_str()) {
            anyhow::bail!("node sync contains a duplicate certificate ID");
        }
        if !certificate_names.insert(certificate.cert_name.to_ascii_lowercase()) {
            anyhow::bail!("node sync contains a duplicate certificate activation name");
        }
    }
    Ok(())
}

fn parse_control_plane_url(value: &str) -> anyhow::Result<reqwest::Url> {
    let url = reqwest::Url::parse(value.trim())?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        anyhow::bail!(
            "SDKWORK_WEB_CONTROL_PLANE_URL must be an HTTP(S) origin without credentials, path, query, or fragment"
        );
    }
    Ok(url)
}

fn validate_node_token(value: &str) -> anyhow::Result<()> {
    if !(16..=4_096).contains(&value.len()) || value.bytes().any(|byte| byte.is_ascii_control()) {
        anyhow::bail!("SDKWORK_WEB_NODE_TOKEN must contain 16..=4096 non-control bytes");
    }
    Ok(())
}

fn required_env_alias(preferred: &str, legacy: &str) -> anyhow::Result<String> {
    read_env_alias(preferred, legacy)?
        .ok_or_else(|| anyhow::anyhow!("{preferred} is required ({legacy} is a legacy alias)"))
}

fn read_env_alias(preferred: &str, legacy: &str) -> anyhow::Result<Option<String>> {
    let preferred_value = read_unicode_env(preferred)?;
    let legacy_value = read_unicode_env(legacy)?;
    resolve_alias_values(preferred, preferred_value, legacy, legacy_value)
}

fn read_unicode_env(name: &str) -> anyhow::Result<Option<String>> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => {
            anyhow::bail!("{name} must contain valid Unicode")
        }
    }
}

fn resolve_alias_values(
    preferred_name: &str,
    preferred_value: Option<String>,
    legacy_name: &str,
    legacy_value: Option<String>,
) -> anyhow::Result<Option<String>> {
    match (preferred_value, legacy_value) {
        (Some(preferred), Some(legacy)) if preferred != legacy => {
            anyhow::bail!("{preferred_name} conflicts with legacy alias {legacy_name}")
        }
        (Some(preferred), _) => Ok(Some(preferred)),
        (None, legacy) => Ok(legacy),
    }
}

fn parse_sync_interval(value: &str) -> anyhow::Result<u64> {
    let interval = value
        .parse::<u64>()
        .map_err(|error| anyhow::anyhow!("invalid SDKWORK_WEB_NODE_SYNC_INTERVAL_SECS: {error}"))?;
    if !(MIN_SYNC_INTERVAL_SECS..=MAX_SYNC_INTERVAL_SECS).contains(&interval) {
        anyhow::bail!(
            "SDKWORK_WEB_NODE_SYNC_INTERVAL_SECS must be between {MIN_SYNC_INTERVAL_SECS} and {MAX_SYNC_INTERVAL_SECS}"
        );
    }
    Ok(interval)
}

fn apply_certificate_bundle(
    edge: &EdgeRuntime,
    certificate: &AgentCertificateBundle,
) -> anyhow::Result<()> {
    use sdkwork_webserver_acme_service::IssuedCertificateMaterial;

    let material = IssuedCertificateMaterial {
        cert_name: certificate.cert_name.clone(),
        cert_type: 0,
        issuer: String::new(),
        subject: certificate.cert_name.clone(),
        san_list: certificate.cert_name.clone(),
        fingerprint: certificate.fingerprint.clone(),
        cert_pem: certificate.fullchain_pem.clone(),
        private_key_pem: certificate.privkey_pem.clone(),
        chain_pem: Some(certificate.fullchain_pem.clone()),
        not_before: String::new(),
        not_after: String::new(),
        cert_path: String::new(),
        key_path: String::new(),
        chain_path: None,
    };
    edge.write_certificate_bundle(&material)
        .map_err(|error| anyhow::anyhow!("write certificate {}: {error}", certificate.cert_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_response_decoder_requires_the_canonical_envelope() {
        let decoded: AgentSyncResponse = decode_resource_response(
            br#"{
                "code": 0,
                "data": {
                    "item": {
                        "serverId": "server-1",
                        "syncVersion": "sv1:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                        "unchanged": true,
                        "nginxConfigs": [],
                        "certificates": []
                    }
                },
                "traceId": "trace-1"
            }"#,
        )
        .expect("canonical SDKWork resource response");
        assert_eq!(decoded.server_id, "server-1");
        assert!(decoded.unchanged);

        assert!(decode_resource_response::<AgentSyncResponse>(
            br#"{"serverId":"server-1","syncVersion":"sv1:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","unchanged":true,"nginxConfigs":[],"certificates":[]}"#,
        )
        .is_err());
        assert!(decode_resource_response::<AgentSyncResponse>(
            br#"{"code":40101,"data":{"item":{"serverId":"server-1","syncVersion":"sv1:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","unchanged":true,"nginxConfigs":[],"certificates":[]}},"traceId":"trace-2"}"#,
        )
        .is_err());
    }

    #[test]
    fn runtime_inputs_are_strict_and_bounded() {
        assert!(parse_control_plane_url("https://control.sdkwork.com").is_ok());
        for invalid in [
            "file:///tmp/control",
            "https://user:secret@control.sdkwork.com",
            "https://control.sdkwork.com/backend",
            "https://control.sdkwork.com?tenant=1",
        ] {
            assert!(parse_control_plane_url(invalid).is_err(), "{invalid}");
        }
        assert!(validate_node_token("0123456789abcdef").is_ok());
        assert!(validate_node_token("short").is_err());
        assert!(parse_sync_interval("1").is_ok());
        assert!(parse_sync_interval("3600").is_ok());
        assert!(parse_sync_interval("0").is_err());
        assert!(parse_sync_interval("3601").is_err());
    }

    #[test]
    fn node_configuration_aliases_are_additive_and_fail_on_conflict() {
        assert_eq!(
            resolve_alias_values(
                "SDKWORK_WEB_NODE_TOKEN",
                Some("preferred".to_string()),
                "SDKWORK_WEB_AGENT_TOKEN",
                None,
            )
            .unwrap(),
            Some("preferred".to_string())
        );
        assert_eq!(
            resolve_alias_values(
                "SDKWORK_WEB_NODE_TOKEN",
                None,
                "SDKWORK_WEB_AGENT_TOKEN",
                Some("legacy".to_string()),
            )
            .unwrap(),
            Some("legacy".to_string())
        );
        assert!(resolve_alias_values(
            "SDKWORK_WEB_NODE_TOKEN",
            Some("left".to_string()),
            "SDKWORK_WEB_AGENT_TOKEN",
            Some("right".to_string()),
        )
        .is_err());
    }

    #[test]
    fn node_sync_manifest_rejects_duplicate_targets_and_bad_fingerprints() {
        let mut manifest = AgentSyncResponse {
            server_id: "node-1".to_string(),
            sync_version: "sv1:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            unchanged: false,
            nginx_configs: vec![sdkwork_webserver_contract::AgentNginxConfigBundle {
                config_id: "config-1".to_string(),
                domain: "Example.com".to_string(),
                config_content: "server {}".to_string(),
                fingerprint: hex::encode(Sha256::digest(b"server {}")),
                version: 1,
            }],
            certificates: Vec::new(),
        };
        validate_manifest_bounds(&manifest).unwrap();

        manifest.nginx_configs[0].fingerprint = "bad".to_string();
        assert!(validate_manifest_bounds(&manifest).is_err());
        manifest.nginx_configs[0].fingerprint = hex::encode(Sha256::digest(b"server {}"));
        let mut duplicate = manifest.nginx_configs[0].clone();
        duplicate.config_id = "config-2".to_string();
        duplicate.domain = "example.COM".to_string();
        manifest.nginx_configs.push(duplicate);
        assert!(validate_manifest_bounds(&manifest).is_err());
    }
}
