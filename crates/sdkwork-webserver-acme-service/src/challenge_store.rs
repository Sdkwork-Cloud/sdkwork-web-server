use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use crate::{AcmeServiceError, AcmeServiceResult};

const CHALLENGE_DIR: &str = ".well-known/acme-challenge";

/// HTTP-01 challenge token store with optional webroot persistence.
#[derive(Default)]
pub struct ChallengeStore {
    tokens: RwLock<HashMap<String, String>>,
}

impl ChallengeStore {
    pub fn register(
        &self,
        webroot: Option<&Path>,
        token: &str,
        key_auth: &str,
    ) -> AcmeServiceResult<()> {
        self.tokens
            .write()
            .map_err(|_| AcmeServiceError::Internal("challenge store lock poisoned".to_string()))?
            .insert(token.to_string(), key_auth.to_string());

        if let Some(root) = webroot {
            let challenge_path = root.join(CHALLENGE_DIR).join(token);
            if let Some(parent) = challenge_path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    AcmeServiceError::Internal(format!("create acme webroot: {error}"))
                })?;
            }
            std::fs::write(&challenge_path, key_auth).map_err(|error| {
                AcmeServiceError::Internal(format!("write acme challenge file: {error}"))
            })?;
        }

        Ok(())
    }

    pub fn lookup(&self, token: &str) -> Option<String> {
        self.tokens
            .read()
            .ok()
            .and_then(|guard| guard.get(token).cloned())
    }

    pub fn clear_token(&self, webroot: Option<&Path>, token: &str) {
        if let Ok(mut guard) = self.tokens.write() {
            guard.remove(token);
        }
        if let Some(root) = webroot {
            let challenge_path = root.join(CHALLENGE_DIR).join(token);
            let _ = std::fs::remove_file(challenge_path);
        }
    }

    pub fn challenge_dir(webroot: &Path) -> PathBuf {
        webroot.join(CHALLENGE_DIR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_lookup_in_memory() {
        let store = ChallengeStore::default();
        store
            .register(None, "token-a", "key-auth")
            .expect("register");
        assert_eq!(store.lookup("token-a").as_deref(), Some("key-auth"));
    }
}
