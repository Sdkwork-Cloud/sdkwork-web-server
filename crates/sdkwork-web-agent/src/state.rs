use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

const STATE_SCHEMA_VERSION: u32 = 1;
const MAX_STATE_BYTES: u64 = 8 * 1024;
const SYNC_VERSION_PREFIX: &str = "sv1:";
const SYNC_VERSION_HEX_BYTES: usize = 64;
const NODE_DAEMON_LOCK_FILE: &str = "sdkwork-web-node-daemon.lock";

pub struct NodeDaemonLock {
    _file: File,
}

impl NodeDaemonLock {
    pub fn acquire(state_path: &Path) -> anyhow::Result<Self> {
        require_absolute_state_path(state_path)?;
        let parent = state_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("node state path has no parent"))?;
        reject_symlink_ancestors(parent)?;
        fs::create_dir_all(parent).context("create node state directory for process lock")?;
        reject_symlink_ancestors(parent)?;

        let lock_path = parent.join(NODE_DAEMON_LOCK_FILE);
        if let Ok(metadata) = fs::symlink_metadata(&lock_path) {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("node daemon lock path must be a regular non-symlink file");
            }
        }
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&lock_path)
            .with_context(|| format!("open node daemon lock {}", lock_path.display()))?;
        let metadata = fs::symlink_metadata(&lock_path)
            .with_context(|| format!("inspect node daemon lock {}", lock_path.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            bail!("node daemon lock path changed during acquisition");
        }
        secure_state_file(&file)?;
        match file.try_lock() {
            Ok(()) => {}
            Err(std::fs::TryLockError::WouldBlock) => {
                bail!("another Web Node Daemon already owns this state directory")
            }
            Err(std::fs::TryLockError::Error(error)) => {
                return Err(error).context("acquire exclusive node daemon process lock")
            }
        }
        sync_directory(parent)?;
        Ok(Self { _file: file })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NodeDaemonState {
    revision: u64,
    desired_sync_version: Option<String>,
    observed_sync_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredNodeDaemonState {
    schema_version: u32,
    revision: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    desired_sync_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    observed_sync_version: Option<String>,
    checksum: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StateChecksumPayload<'a> {
    schema_version: u32,
    revision: u64,
    desired_sync_version: &'a Option<String>,
    observed_sync_version: &'a Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyAgentState {
    last_sync_version: Option<String>,
}

impl NodeDaemonState {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        require_absolute_state_path(path)?;
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(error) => return Err(error).context("inspect Web Node Daemon state file"),
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            bail!("Web Node Daemon state path must be a regular non-symlink file");
        }
        if metadata.len() == 0 || metadata.len() > MAX_STATE_BYTES {
            bail!("Web Node Daemon state file must be within 1..={MAX_STATE_BYTES} bytes");
        }

        let mut bytes = Vec::with_capacity(metadata.len() as usize + 1);
        File::open(path)
            .context("open Web Node Daemon state file")?
            .take(MAX_STATE_BYTES + 1)
            .read_to_end(&mut bytes)
            .context("read Web Node Daemon state file")?;
        if bytes.len() as u64 > MAX_STATE_BYTES {
            bail!("Web Node Daemon state file grew beyond {MAX_STATE_BYTES} bytes while reading");
        }

        match serde_json::from_slice::<StoredNodeDaemonState>(&bytes) {
            Ok(stored) => Self::from_stored(stored),
            Err(stored_error) => {
                let legacy = serde_json::from_slice::<LegacyAgentState>(&bytes).map_err(|_| {
                    anyhow::anyhow!("parse Web Node Daemon state file: {stored_error}")
                })?;
                Self::from_legacy(legacy)
            }
        }
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        require_absolute_state_path(path)?;
        self.validate()?;
        let parent = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Web Node Daemon state path has no parent"))?;
        reject_symlink_ancestors(parent)?;
        fs::create_dir_all(parent).context("create Web Node Daemon state directory")?;
        reject_symlink_ancestors(parent)?;
        if let Ok(metadata) = fs::symlink_metadata(path) {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("Web Node Daemon state target must be a regular non-symlink file");
            }
        }

        let stored = self.to_stored()?;
        let mut payload =
            serde_json::to_vec_pretty(&stored).context("serialize Web Node Daemon state")?;
        payload.push(b'\n');
        if payload.len() as u64 > MAX_STATE_BYTES {
            bail!("serialized Web Node Daemon state exceeds {MAX_STATE_BYTES} bytes");
        }

        let mut staged = NamedTempFile::new_in(parent).context("stage Web Node Daemon state")?;
        staged
            .write_all(&payload)
            .and_then(|_| staged.flush())
            .and_then(|_| staged.as_file().sync_all())
            .context("write Web Node Daemon state")?;
        secure_state_file(staged.as_file())?;
        staged
            .persist(path)
            .map_err(|error| error.error)
            .context("activate Web Node Daemon state")?;
        sync_directory(parent)?;
        Ok(())
    }

    pub fn observed_sync_version(&self) -> Option<&str> {
        self.observed_sync_version.as_deref()
    }

    pub fn desired_sync_version(&self) -> Option<&str> {
        self.desired_sync_version.as_deref()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn is_pending(&self) -> bool {
        self.desired_sync_version != self.observed_sync_version
    }

    pub fn with_desired(&self, sync_version: &str) -> anyhow::Result<Self> {
        validate_sync_version(sync_version)?;
        Ok(Self {
            revision: self
                .revision
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("Web Node Daemon state revision exhausted"))?,
            desired_sync_version: Some(sync_version.to_string()),
            observed_sync_version: self.observed_sync_version.clone(),
        })
    }

    pub fn with_observed(&self, sync_version: &str) -> anyhow::Result<Self> {
        validate_sync_version(sync_version)?;
        if self.desired_sync_version.as_deref() != Some(sync_version) {
            bail!("cannot observe a sync generation that is not the durable desired generation");
        }
        Ok(Self {
            revision: self
                .revision
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("Web Node Daemon state revision exhausted"))?,
            desired_sync_version: Some(sync_version.to_string()),
            observed_sync_version: Some(sync_version.to_string()),
        })
    }

    fn from_stored(stored: StoredNodeDaemonState) -> anyhow::Result<Self> {
        if stored.schema_version != STATE_SCHEMA_VERSION {
            bail!(
                "unsupported Web Node Daemon state schema version: {}",
                stored.schema_version
            );
        }
        let expected = checksum(
            stored.schema_version,
            stored.revision,
            &stored.desired_sync_version,
            &stored.observed_sync_version,
        )?;
        if stored.checksum != expected {
            bail!("Web Node Daemon state checksum mismatch");
        }
        let state = Self {
            revision: stored.revision,
            desired_sync_version: stored.desired_sync_version,
            observed_sync_version: stored.observed_sync_version,
        };
        state.validate()?;
        Ok(state)
    }

    fn from_legacy(legacy: LegacyAgentState) -> anyhow::Result<Self> {
        if let Some(sync_version) = legacy.last_sync_version.as_deref() {
            validate_sync_version(sync_version)?;
        }
        Ok(Self {
            revision: u64::from(legacy.last_sync_version.is_some()),
            desired_sync_version: legacy.last_sync_version.clone(),
            observed_sync_version: legacy.last_sync_version,
        })
    }

    fn to_stored(&self) -> anyhow::Result<StoredNodeDaemonState> {
        Ok(StoredNodeDaemonState {
            schema_version: STATE_SCHEMA_VERSION,
            revision: self.revision,
            desired_sync_version: self.desired_sync_version.clone(),
            observed_sync_version: self.observed_sync_version.clone(),
            checksum: checksum(
                STATE_SCHEMA_VERSION,
                self.revision,
                &self.desired_sync_version,
                &self.observed_sync_version,
            )?,
        })
    }

    fn validate(&self) -> anyhow::Result<()> {
        if let Some(version) = self.desired_sync_version.as_deref() {
            validate_sync_version(version)?;
        }
        if let Some(version) = self.observed_sync_version.as_deref() {
            validate_sync_version(version)?;
            if self.desired_sync_version.is_none() {
                bail!("observed sync generation requires a desired generation");
            }
        }
        Ok(())
    }
}

pub fn resolve_state_path() -> anyhow::Result<PathBuf> {
    let path = if let Some(path) = read_path_env_alias(
        "SDKWORK_WEB_NODE_STATE_PATH",
        "SDKWORK_WEB_AGENT_STATE_PATH",
    )? {
        PathBuf::from(path)
    } else {
        let root = if let Some(path) =
            read_path_env_alias("SDKWORK_WEB_NODE_STATE_DIR", "SDKWORK_WEB_AGENT_STATE_DIR")?
        {
            PathBuf::from(path)
        } else if let Some(path) = std::env::var_os("SDKWORK_WEB_EDGE_ROOT") {
            PathBuf::from(path)
        } else {
            default_edge_state_root()?
        };
        root.join("sdkwork-web-agent-state.json")
    };
    require_absolute_state_path(&path)?;
    Ok(path)
}

fn read_path_env_alias(preferred: &str, legacy: &str) -> anyhow::Result<Option<OsString>> {
    resolve_path_alias_values(
        preferred,
        std::env::var_os(preferred),
        legacy,
        std::env::var_os(legacy),
    )
}

fn resolve_path_alias_values(
    preferred_name: &str,
    preferred_value: Option<OsString>,
    legacy_name: &str,
    legacy_value: Option<OsString>,
) -> anyhow::Result<Option<OsString>> {
    match (preferred_value, legacy_value) {
        (Some(preferred), Some(legacy)) if preferred != legacy => {
            anyhow::bail!("{preferred_name} conflicts with legacy alias {legacy_name}")
        }
        (Some(preferred), _) => Ok(Some(preferred)),
        (None, legacy) => Ok(legacy),
    }
}

fn default_edge_state_root() -> anyhow::Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let program_data = std::env::var_os("PROGRAMDATA")
            .ok_or_else(|| anyhow::anyhow!("PROGRAMDATA is required for Web Node Daemon state"))?;
        return Ok(PathBuf::from(program_data)
            .join("sdkwork")
            .join("web")
            .join("Data")
            .join("edge"));
    }
    #[cfg(target_os = "macos")]
    {
        return Ok(PathBuf::from(
            "/Library/Application Support/sdkwork/web/Data/edge",
        ));
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Ok(PathBuf::from("/var/lib/sdkwork/web/edge"))
    }
    #[cfg(not(any(unix, target_os = "windows")))]
    {
        bail!("SDKWORK_WEB_AGENT_STATE_DIR is required on this platform")
    }
}

fn validate_sync_version(sync_version: &str) -> anyhow::Result<()> {
    let digest = sync_version
        .strip_prefix(SYNC_VERSION_PREFIX)
        .ok_or_else(|| anyhow::anyhow!("sync version must use the sv1 SHA-256 format"))?;
    if digest.len() != SYNC_VERSION_HEX_BYTES
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        bail!("sync version must contain exactly 64 lowercase hexadecimal bytes");
    }
    Ok(())
}

fn checksum(
    schema_version: u32,
    revision: u64,
    desired_sync_version: &Option<String>,
    observed_sync_version: &Option<String>,
) -> anyhow::Result<String> {
    let payload = StateChecksumPayload {
        schema_version,
        revision,
        desired_sync_version,
        observed_sync_version,
    };
    let bytes = serde_json::to_vec(&payload).context("serialize state checksum payload")?;
    Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
}

fn require_absolute_state_path(path: &Path) -> anyhow::Result<()> {
    if !path.is_absolute() {
        bail!("Web Node Daemon state path must be absolute");
    }
    Ok(())
}

fn reject_symlink_ancestors(path: &Path) -> anyhow::Result<()> {
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                bail!(
                    "Web Node Daemon state directory contains a symlink ancestor: {}",
                    ancestor.display()
                );
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "inspect Web Node Daemon state ancestor {}",
                        ancestor.display()
                    )
                });
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn secure_state_file(file: &File) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = file
        .metadata()
        .context("read staged Web Node Daemon state permissions")?
        .permissions();
    permissions.set_mode(0o600);
    file.set_permissions(permissions)
        .context("secure staged Web Node Daemon state")
}

#[cfg(not(unix))]
fn secure_state_file(_file: &File) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> anyhow::Result<()> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .context("sync Web Node Daemon state directory")
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::TempDir;

    use super::*;

    fn version(byte: char) -> String {
        format!("sv1:{}", byte.to_string().repeat(64))
    }

    #[test]
    fn desired_and_observed_generations_survive_restart_without_false_success() {
        let directory = TempDir::new().expect("state tempdir");
        let path = directory.path().join("agent-state.json");
        let first = version('a');
        let second = version('b');

        let stable = NodeDaemonState::default()
            .with_desired(&first)
            .expect("first desired")
            .with_observed(&first)
            .expect("first observed");
        stable.save(&path).expect("save stable state");
        let pending = NodeDaemonState::load(&path)
            .expect("load stable state")
            .with_desired(&second)
            .expect("second desired");
        pending.save(&path).expect("save pending state");

        let recovered = NodeDaemonState::load(&path).expect("load pending state");
        assert_eq!(recovered.observed_sync_version(), Some(first.as_str()));
        assert_eq!(recovered.desired_sync_version(), Some(second.as_str()));
        assert!(recovered.is_pending());
        let completed = recovered
            .with_observed(&second)
            .expect("complete desired generation");
        assert!(!completed.is_pending());
        assert_eq!(completed.revision(), 4);
    }

    #[test]
    fn state_checksum_corruption_and_invalid_transitions_fail_closed() {
        let directory = TempDir::new().expect("state tempdir");
        let path = directory.path().join("agent-state.json");
        let current = version('c');
        let next = version('d');
        let state = NodeDaemonState::default()
            .with_desired(&current)
            .expect("desired");
        state.save(&path).expect("save state");
        let mut raw = fs::read_to_string(&path).expect("read state");
        raw = raw.replace(&current, &next);
        fs::write(&path, raw).expect("corrupt state");
        assert!(NodeDaemonState::load(&path).is_err());
        assert!(state.with_observed(&next).is_err());
        assert!(state.with_desired("not-a-version").is_err());
    }

    #[test]
    fn legacy_state_migrates_but_unknown_or_oversized_state_is_rejected() {
        let directory = TempDir::new().expect("state tempdir");
        let path = directory.path().join("agent-state.json");
        let current = version('e');
        fs::write(&path, format!(r#"{{"lastSyncVersion":"{current}"}}"#))
            .expect("write legacy state");
        let migrated = NodeDaemonState::load(&path).expect("migrate legacy state");
        assert_eq!(migrated.observed_sync_version(), Some(current.as_str()));
        assert!(!migrated.is_pending());

        fs::write(&path, r#"{"lastSyncVersion":null,"unknown":true}"#)
            .expect("write unknown state");
        assert!(NodeDaemonState::load(&path).is_err());
        let mut oversized = File::create(&path).expect("create oversized state");
        oversized
            .write_all(&vec![b'x'; MAX_STATE_BYTES as usize + 1])
            .expect("write oversized state");
        assert!(NodeDaemonState::load(&path).is_err());
    }

    #[test]
    fn node_state_path_aliases_are_additive_and_fail_on_conflict() {
        assert_eq!(
            resolve_path_alias_values(
                "SDKWORK_WEB_NODE_STATE_PATH",
                Some(OsString::from("preferred")),
                "SDKWORK_WEB_AGENT_STATE_PATH",
                None,
            )
            .unwrap(),
            Some(OsString::from("preferred"))
        );
        assert_eq!(
            resolve_path_alias_values(
                "SDKWORK_WEB_NODE_STATE_PATH",
                None,
                "SDKWORK_WEB_AGENT_STATE_PATH",
                Some(OsString::from("legacy")),
            )
            .unwrap(),
            Some(OsString::from("legacy"))
        );
        assert!(resolve_path_alias_values(
            "SDKWORK_WEB_NODE_STATE_PATH",
            Some(OsString::from("left")),
            "SDKWORK_WEB_AGENT_STATE_PATH",
            Some(OsString::from("right")),
        )
        .is_err());
    }

    #[test]
    fn node_daemon_lock_is_exclusive_and_released_on_drop() {
        let directory = TempDir::new().expect("state tempdir");
        let state_path = directory.path().join("node-state.json");

        let first = NodeDaemonLock::acquire(&state_path).expect("first process lock");
        assert!(NodeDaemonLock::acquire(&state_path).is_err());
        let lock_path = directory.path().join(NODE_DAEMON_LOCK_FILE);
        assert!(lock_path.is_file());
        assert_eq!(fs::metadata(&lock_path).expect("lock metadata").len(), 0);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&lock_path)
                    .expect("lock metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }

        drop(first);
        NodeDaemonLock::acquire(&state_path).expect("lock after owner drop");
    }

    #[cfg(unix)]
    #[test]
    fn state_target_and_ancestor_symlinks_are_rejected() {
        use std::os::unix::fs::symlink;

        let directory = TempDir::new().expect("state tempdir");
        let real = directory.path().join("real.json");
        fs::write(&real, "{}").expect("write real target");
        let linked = directory.path().join("linked.json");
        symlink(&real, &linked).expect("create state symlink");
        assert!(NodeDaemonState::load(&linked).is_err());
        assert!(NodeDaemonState::default().save(&linked).is_err());

        let lock_target = directory.path().join("lock-target");
        fs::write(&lock_target, "").expect("write lock target");
        let lock_path = directory.path().join(NODE_DAEMON_LOCK_FILE);
        symlink(&lock_target, &lock_path).expect("create lock symlink");
        assert!(NodeDaemonLock::acquire(&directory.path().join("other-state.json")).is_err());
    }
}
