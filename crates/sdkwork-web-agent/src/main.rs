//! SDKWork Web Node Daemon using the legacy v3 Agent API compatibility contract.

#[path = "state.rs"]
mod state;

use std::collections::HashSet;
use std::time::Duration;

use sdkwork_utils_rust::crypto::sha256_hash;
use sdkwork_web_backend_sdk::{
    AgentHeartbeatRequest as SdkAgentHeartbeatRequest,
    AgentHeartbeatResponse as SdkAgentHeartbeatResponse, AgentSyncResponse as SdkAgentSyncResponse,
    SdkworkBackendClient, SdkworkConfig,
};
use sdkwork_webserver_contract::{
    AgentCertificateBundle, AgentHeartbeatResponse, AgentNginxConfigBundle, AgentSyncResponse,
};
use sdkwork_webserver_edge_runtime::EdgeRuntime;
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
    control_plane: String,
    node_token: String,
    interval_secs: u64,
}

struct NodeDaemonSdkClients {
    heartbeat: SdkworkBackendClient,
    sync: SdkworkBackendClient,
}

impl NodeDaemonSdkClients {
    fn new(runtime: &NodeDaemonRuntimeConfig) -> anyhow::Result<Self> {
        Ok(Self {
            heartbeat: build_backend_sdk_client(runtime, MAX_HEARTBEAT_RESPONSE_BYTES)?,
            sync: build_backend_sdk_client(runtime, MAX_SYNC_RESPONSE_BYTES)?,
        })
    }
}

fn build_backend_sdk_client(
    runtime: &NodeDaemonRuntimeConfig,
    maximum_response_bytes: usize,
) -> anyhow::Result<SdkworkBackendClient> {
    let mut config = SdkworkConfig::new(runtime.control_plane.clone());
    config.timeout_ms = HTTP_TIMEOUT_SECS * 1_000;
    config.max_response_body_bytes = maximum_response_bytes;
    let client = SdkworkBackendClient::new(config)?;
    client.set_agent_token(runtime.node_token.clone());
    Ok(client)
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
    let clients = NodeDaemonSdkClients::new(&runtime)?;

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
        if let Err(error) = sync_once(&edge, &clients, &state_path, &mut local_state).await {
            warn!(error = %error, "node sync cycle failed");
        }
        tokio::time::sleep(Duration::from_secs(runtime.interval_secs)).await;
    }
}

async fn sync_once(
    edge: &EdgeRuntime,
    clients: &NodeDaemonSdkClients,
    state_path: &std::path::Path,
    local_state: &mut NodeDaemonState,
) -> anyhow::Result<()> {
    let heartbeat = SdkAgentHeartbeatRequest {
        agent_version: Some(NODE_DAEMON_VERSION.to_string()),
        nginx_enabled: Some(edge.config().nginx_enabled),
        active_configs: None,
        last_sync_version: local_state.observed_sync_version().map(str::to_string),
    };
    let heartbeat_ack =
        map_heartbeat_response(clients.heartbeat.agent().heartbeat(&heartbeat).await?)?;
    if heartbeat_ack.server_id.trim().is_empty() {
        anyhow::bail!("control-plane heartbeat acknowledgement has an empty serverId");
    }
    if heartbeat_ack.status != 1 {
        anyhow::bail!("control-plane heartbeat acknowledgement did not mark the node active");
    }

    let manifest = map_sync_response(
        clients
            .sync
            .agent()
            .retrieve(local_state.observed_sync_version())
            .await?,
    )?;
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

fn map_heartbeat_response(
    response: SdkAgentHeartbeatResponse,
) -> anyhow::Result<AgentHeartbeatResponse> {
    Ok(AgentHeartbeatResponse {
        server_id: response.server_id,
        status: i32::try_from(response.status)
            .map_err(|_| anyhow::anyhow!("heartbeat status is outside the i32 range"))?,
        acknowledged_at: response.acknowledged_at,
    })
}

fn map_sync_response(response: SdkAgentSyncResponse) -> anyhow::Result<AgentSyncResponse> {
    let nginx_configs = response
        .nginx_configs
        .into_iter()
        .map(|config| {
            Ok(AgentNginxConfigBundle {
                config_id: config.config_id,
                domain: config.domain,
                config_content: config.config_content,
                fingerprint: config.fingerprint,
                version: config
                    .version
                    .parse::<i64>()
                    .map_err(|error| anyhow::anyhow!("invalid node sync Nginx version: {error}"))?,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let certificates = response
        .certificates
        .into_iter()
        .map(|certificate| AgentCertificateBundle {
            certificate_id: certificate.certificate_id,
            cert_name: certificate.cert_name,
            fingerprint: certificate.fingerprint,
            fullchain_pem: certificate.fullchain_pem,
            privkey_pem: certificate.privkey_pem,
        })
        .collect();
    Ok(AgentSyncResponse {
        server_id: response.server_id,
        sync_version: response.sync_version,
        unchanged: response.unchanged,
        nginx_configs,
        certificates,
    })
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
        let fingerprint = sha256_hash(config.config_content.as_bytes());
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

fn parse_control_plane_url(value: &str) -> anyhow::Result<String> {
    let url = url::Url::parse(value.trim())?;
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
    Ok(url.to_string())
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
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::sync::oneshot;

    use super::*;

    async fn serve_once(
        status: &str,
        content_length: usize,
        body: &'static [u8],
    ) -> (String, oneshot::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind SDK mock server");
        let address = listener.local_addr().expect("SDK mock address");
        let (request_sender, request_receiver) = oneshot::channel();
        let status = status.to_string();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept SDK request");
            let mut request = vec![0_u8; 16 * 1024];
            let bytes_read = stream.read(&mut request).await.expect("read SDK request");
            request.truncate(bytes_read);
            let _ = request_sender.send(String::from_utf8_lossy(&request).to_string());
            let headers = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {content_length}\r\nConnection: close\r\n\r\n"
            );
            stream
                .write_all(headers.as_bytes())
                .await
                .expect("write SDK response headers");
            stream
                .write_all(body)
                .await
                .expect("write SDK response body");
            stream.shutdown().await.expect("close SDK response");
        });
        (format!("http://{address}/"), request_receiver)
    }

    fn heartbeat_request() -> SdkAgentHeartbeatRequest {
        SdkAgentHeartbeatRequest {
            agent_version: Some("0.1.0".to_string()),
            nginx_enabled: Some(true),
            active_configs: None,
            last_sync_version: None,
        }
    }

    #[tokio::test]
    async fn generated_sdk_applies_agent_token_unwraps_envelope_and_enforces_body_limit() {
        let body = br#"{"code":0,"data":{"item":{"serverId":"server-1","status":1,"acknowledgedAt":"2026-07-20T00:00:00Z"}},"traceId":"trace-1"}"#;
        let (control_plane, request_receiver) = serve_once("200 OK", body.len(), body).await;
        let runtime = NodeDaemonRuntimeConfig {
            control_plane,
            node_token: "0123456789abcdef".to_string(),
            interval_secs: 30,
        };
        let client = build_backend_sdk_client(&runtime, MAX_HEARTBEAT_RESPONSE_BYTES)
            .expect("build generated backend SDK client");
        let response = client
            .agent()
            .heartbeat(&heartbeat_request())
            .await
            .expect("decode bounded SDKWork resource envelope");
        assert_eq!(response.server_id, "server-1");
        let request = request_receiver.await.expect("captured SDK request");
        assert!(request.starts_with("POST /backend/v3/api/agent/heartbeat HTTP/1.1"));
        assert!(request
            .to_ascii_lowercase()
            .contains("x-sdkwork-agent-token: 0123456789abcdef"));

        let (control_plane, _) = serve_once("200 OK", MAX_HEARTBEAT_RESPONSE_BYTES + 1, b"").await;
        let runtime = NodeDaemonRuntimeConfig {
            control_plane,
            node_token: "0123456789abcdef".to_string(),
            interval_secs: 30,
        };
        let client = build_backend_sdk_client(&runtime, MAX_HEARTBEAT_RESPONSE_BYTES)
            .expect("build bounded generated backend SDK client");
        let error = client
            .agent()
            .heartbeat(&heartbeat_request())
            .await
            .expect_err("oversized SDK response must fail closed");
        assert!(error
            .to_string()
            .contains("response body exceeds 65536 bytes"));
    }

    #[test]
    fn generated_sdk_models_map_to_the_domain_contract() {
        let decoded = map_sync_response(SdkAgentSyncResponse {
            server_id: "server-1".to_string(),
            sync_version: "sv1:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            unchanged: true,
            nginx_configs: Vec::new(),
            certificates: Vec::new(),
        })
        .expect("generated SDK response mapping");
        assert_eq!(decoded.server_id, "server-1");
        assert!(decoded.unchanged);
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
                fingerprint: sha256_hash(b"server {}"),
                version: 1,
            }],
            certificates: Vec::new(),
        };
        validate_manifest_bounds(&manifest).unwrap();

        manifest.nginx_configs[0].fingerprint = "bad".to_string();
        assert!(validate_manifest_bounds(&manifest).is_err());
        manifest.nginx_configs[0].fingerprint = sha256_hash(b"server {}");
        let mut duplicate = manifest.nginx_configs[0].clone();
        duplicate.config_id = "config-2".to_string();
        duplicate.domain = "example.COM".to_string();
        manifest.nginx_configs.push(duplicate);
        assert!(validate_manifest_bounds(&manifest).is_err());
    }
}
