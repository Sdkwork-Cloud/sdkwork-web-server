use std::sync::LazyLock;

use serde_json::Value;

use crate::ConfigDiagnostic;

use super::{
    validate_website_runtime_descriptor, website_runtime_descriptor_sha256,
    CompiledWebsiteRuntimeDescriptor, WebsiteRuntimeDescriptor, WebsiteRuntimeDescriptorError,
};

pub const MAX_WEBSITE_RUNTIME_DESCRIPTOR_BYTES: usize = 4 * 1024 * 1024;
const MAX_SCHEMA_DIAGNOSTICS: usize = 64;
const SCHEMA: &str =
    include_str!("../../../../specs/sdkwork.website-runtime.descriptor.schema.json");
static SCHEMA_VALIDATOR: LazyLock<Result<jsonschema::Validator, String>> = LazyLock::new(|| {
    let schema: Value = serde_json::from_str(SCHEMA).map_err(|error| error.to_string())?;
    jsonschema::draft202012::new(&schema).map_err(|error| error.to_string())
});

pub fn compile_website_runtime_descriptor(
    bytes: &[u8],
) -> Result<CompiledWebsiteRuntimeDescriptor, WebsiteRuntimeDescriptorError> {
    if bytes.len() > MAX_WEBSITE_RUNTIME_DESCRIPTOR_BYTES {
        return Err(WebsiteRuntimeDescriptorError::TooLarge {
            actual_bytes: bytes.len(),
            maximum_bytes: MAX_WEBSITE_RUNTIME_DESCRIPTOR_BYTES,
        });
    }
    let instance: Value = serde_json::from_slice(bytes)?;
    validate_schema(&instance)?;
    let descriptor: WebsiteRuntimeDescriptor = serde_json::from_value(instance)?;
    let calculated = website_runtime_descriptor_sha256(&descriptor)?;
    if descriptor.descriptor_sha256 != calculated {
        return Err(WebsiteRuntimeDescriptorError::HashMismatch {
            expected: descriptor.descriptor_sha256.clone(),
            calculated,
        });
    }
    validate_website_runtime_descriptor(&descriptor)?;
    Ok(CompiledWebsiteRuntimeDescriptor::compile(
        descriptor, calculated,
    ))
}

fn validate_schema(instance: &Value) -> Result<(), WebsiteRuntimeDescriptorError> {
    let validator = SCHEMA_VALIDATOR
        .as_ref()
        .map_err(|error| WebsiteRuntimeDescriptorError::InvalidSchema(error.clone()))?;
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
        Err(WebsiteRuntimeDescriptorError::Validation { diagnostics })
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
