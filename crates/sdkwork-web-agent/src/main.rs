//! SDKWork Web edge agent: syncs nginx configs and certificate bundles to the local node.

mod state;

use std::time::Duration;

use sdkwork_webserver_contract::{
    AgentCertificateBundle, AgentHeartbeatRequest, AgentSyncResponse,
};
use sdkwork_webserver_edge_runtime::EdgeRuntime;
use state::{resolve_state_path, AgentLocalState};
use tracing::{info, warn};

const AGENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let edge = EdgeRuntime::from_env()?;
    let interval_secs = std::env::var("SDKWORK_WEB_AGENT_SYNC_INTERVAL_SECS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(30);

    info!(
        interval_secs,
        nginx_enabled = edge.config().nginx_enabled,
        "sdkwork-web-agent started"
    );

    loop {
        if let Err(error) = sync_once(&edge).await {
            warn!(error = %error, "agent sync cycle failed");
        }
        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
    }
}

async fn sync_once(edge: &EdgeRuntime) -> anyhow::Result<()> {
    let control_plane = std::env::var("SDKWORK_WEB_CONTROL_PLANE_URL")
        .map_err(|_| anyhow::anyhow!("SDKWORK_WEB_CONTROL_PLANE_URL is required"))?;
    let agent_token = std::env::var("SDKWORK_WEB_AGENT_TOKEN")
        .map_err(|_| anyhow::anyhow!("SDKWORK_WEB_AGENT_TOKEN is required"))?;

    let state_path = resolve_state_path();
    let mut local_state = AgentLocalState::load(&state_path);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    let heartbeat_url = format!("{control_plane}/backend/v3/api/agent/heartbeat");
    let heartbeat = AgentHeartbeatRequest {
        agent_version: Some(AGENT_VERSION.to_string()),
        nginx_enabled: Some(edge.config().nginx_enabled),
        active_configs: None,
        last_sync_version: local_state.last_sync_version.clone(),
    };
    client
        .post(heartbeat_url)
        .header("X-SDKWork-Agent-Token", &agent_token)
        .json(&heartbeat)
        .send()
        .await?
        .error_for_status()?;

    let mut sync_url = format!("{control_plane}/backend/v3/api/agent/sync");
    if let Some(last_sync_version) = local_state.last_sync_version.as_deref() {
        sync_url = format!(
            "{sync_url}?ifSyncVersion={}",
            urlencoding::encode(last_sync_version)
        );
    }

    let response = client
        .get(sync_url)
        .header("X-SDKWork-Agent-Token", &agent_token)
        .send()
        .await?
        .error_for_status()?;
    let manifest: AgentSyncResponse = response.json().await?;

    if manifest.unchanged {
        info!(
            server_id = %manifest.server_id,
            sync_version = %manifest.sync_version,
            "agent sync manifest unchanged"
        );
        return Ok(());
    }

    info!(
        server_id = %manifest.server_id,
        sync_version = %manifest.sync_version,
        nginx_configs = manifest.nginx_configs.len(),
        certificates = manifest.certificates.len(),
        "agent sync manifest received"
    );

    for config in &manifest.nginx_configs {
        edge.deploy_site_config(&config.domain, &config.config_content)
            .map_err(|error| anyhow::anyhow!("deploy nginx site {}: {error}", config.domain))?;
    }

    for certificate in &manifest.certificates {
        apply_certificate_bundle(edge, certificate)?;
    }

    edge.reload()?;

    local_state.last_sync_version = Some(manifest.sync_version.clone());
    local_state.save(&state_path)?;

    Ok(())
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
