use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AgentLocalState {
    #[serde(rename = "lastSyncVersion", skip_serializing_if = "Option::is_none")]
    pub last_sync_version: Option<String>,
}

impl AgentLocalState {
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let payload = serde_json::to_string_pretty(self)?;
        fs::write(path, payload)?;
        Ok(())
    }
}

pub fn resolve_state_path() -> PathBuf {
    if let Ok(path) = std::env::var("SDKWORK_WEB_AGENT_STATE_PATH") {
        return PathBuf::from(path);
    }

    let base = std::env::var("SDKWORK_WEB_AGENT_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());

    base.join("sdkwork-web-agent-state.json")
}
