use crate::canonical_json::canonical_sha256_excluding_field;

use super::{TlsAssignmentSnapshot, TlsRuntimeSnapshotError};

pub fn tls_assignment_snapshot_sha256(
    snapshot: &TlsAssignmentSnapshot,
) -> Result<String, TlsRuntimeSnapshotError> {
    canonical_sha256_excluding_field(snapshot, "snapshotSha256")
        .map_err(TlsRuntimeSnapshotError::Json)
}
