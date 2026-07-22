use std::collections::HashMap;

use crate::website_runtime::normalize_website_hostname;

use super::{InvalidSniServerName, TlsAssignmentSnapshot, TlsCertificateAssignment};

#[derive(Debug)]
pub struct CompiledTlsAssignmentSnapshot {
    snapshot: TlsAssignmentSnapshot,
    snapshot_sha256: String,
    exact_server_names: HashMap<String, usize>,
    wildcard_server_names: Vec<(String, usize)>,
}

impl CompiledTlsAssignmentSnapshot {
    pub(crate) fn compile(snapshot: TlsAssignmentSnapshot, snapshot_sha256: String) -> Self {
        let mut exact_server_names = HashMap::new();
        let mut wildcard_server_names = Vec::new();
        for (assignment_index, assignment) in snapshot.assignments.iter().enumerate() {
            for server_name in &assignment.server_names {
                if let Some(suffix) = server_name.strip_prefix("*.") {
                    wildcard_server_names.push((suffix.to_owned(), assignment_index));
                } else {
                    exact_server_names.insert(server_name.clone(), assignment_index);
                }
            }
        }
        wildcard_server_names.sort_unstable_by(|left, right| {
            right
                .0
                .len()
                .cmp(&left.0.len())
                .then_with(|| left.0.cmp(&right.0))
        });
        Self {
            snapshot,
            snapshot_sha256,
            exact_server_names,
            wildcard_server_names,
        }
    }

    pub fn snapshot(&self) -> &TlsAssignmentSnapshot {
        &self.snapshot
    }

    pub fn snapshot_sha256(&self) -> &str {
        &self.snapshot_sha256
    }

    pub fn select_assignment(
        &self,
        server_name: &str,
    ) -> Result<Option<&TlsCertificateAssignment>, InvalidSniServerName> {
        let server_name = normalize_website_hostname(server_name).ok_or(InvalidSniServerName)?;
        if server_name.starts_with("*.") {
            return Err(InvalidSniServerName);
        }
        if let Some(index) = self.exact_server_names.get(&server_name) {
            return Ok(self.snapshot.assignments.get(*index));
        }
        Ok(self
            .wildcard_server_names
            .iter()
            .find(|(suffix, _)| wildcard_matches(suffix, &server_name))
            .and_then(|(_, index)| self.snapshot.assignments.get(*index)))
    }
}

fn wildcard_matches(suffix: &str, server_name: &str) -> bool {
    server_name
        .strip_suffix(suffix)
        .is_some_and(|prefix| prefix.ends_with('.') && !prefix[..prefix.len() - 1].contains('.'))
}
