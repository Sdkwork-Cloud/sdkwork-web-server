use serde_json::Value;

use crate::ConfigDiagnostic;

use super::{
    tls_assignment_snapshot_sha256, validate_tls_assignment_snapshot,
    CompiledTlsAssignmentSnapshot, TlsAssignmentSnapshot, TlsRuntimeSnapshotError,
};

pub const MAX_TLS_RUNTIME_SNAPSHOT_BYTES: usize = 2 * 1024 * 1024;
const MAX_SCHEMA_DIAGNOSTICS: usize = 64;
const SCHEMA: &str = include_str!("../../../../specs/sdkwork.tls-runtime.snapshot.schema.json");

pub fn compile_tls_assignment_snapshot(
    bytes: &[u8],
) -> Result<CompiledTlsAssignmentSnapshot, TlsRuntimeSnapshotError> {
    if bytes.len() > MAX_TLS_RUNTIME_SNAPSHOT_BYTES {
        return Err(TlsRuntimeSnapshotError::TooLarge {
            actual_bytes: bytes.len(),
            maximum_bytes: MAX_TLS_RUNTIME_SNAPSHOT_BYTES,
        });
    }
    let instance: Value = serde_json::from_slice(bytes)?;
    validate_schema(&instance)?;
    let snapshot: TlsAssignmentSnapshot = serde_json::from_value(instance)?;
    let calculated = tls_assignment_snapshot_sha256(&snapshot)?;
    if snapshot.snapshot_sha256 != calculated {
        return Err(TlsRuntimeSnapshotError::HashMismatch {
            expected: snapshot.snapshot_sha256.clone(),
            calculated,
        });
    }
    validate_tls_assignment_snapshot(&snapshot)?;
    Ok(CompiledTlsAssignmentSnapshot::compile(snapshot, calculated))
}

fn validate_schema(instance: &Value) -> Result<(), TlsRuntimeSnapshotError> {
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|error| TlsRuntimeSnapshotError::InvalidSchema(error.to_string()))?;
    let validator = jsonschema::draft202012::new(&schema)
        .map_err(|error| TlsRuntimeSnapshotError::InvalidSchema(error.to_string()))?;
    let diagnostics = validator
        .iter_errors(instance)
        .take(MAX_SCHEMA_DIAGNOSTICS)
        .map(|error| {
            ConfigDiagnostic::new(
                error.instance_path().as_str(),
                truncate_diagnostic(&error.to_string()),
            )
        })
        .collect::<Vec<_>>();
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(TlsRuntimeSnapshotError::Validation { diagnostics })
    }
}

fn truncate_diagnostic(message: &str) -> String {
    const MAX_DIAGNOSTIC_BYTES: usize = 512;
    if message.len() <= MAX_DIAGNOSTIC_BYTES {
        return message.to_owned();
    }
    let mut end = MAX_DIAGNOSTIC_BYTES;
    while !message.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", &message[..end])
}
