use std::{fs, path::Path};

use serde_json::Value;

use super::{
    validate_webserver_config, CompiledWebServerApp, ConfigDiagnostic, WebServerAppConfig,
    WebServerConfigError,
};

pub const MAX_CONFIG_BYTES: u64 = 1024 * 1024;
const MAX_SCHEMA_DIAGNOSTICS: usize = 64;
const SCHEMA: &str = include_str!("../../../../specs/sdkwork.webserver.config.schema.json");

pub fn load_and_compile_webserver_config(
    path: impl AsRef<Path>,
) -> Result<CompiledWebServerApp, WebServerConfigError> {
    let path = path.as_ref();
    let metadata = fs::metadata(path).map_err(|source| WebServerConfigError::Inspect {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.len() > MAX_CONFIG_BYTES {
        return Err(WebServerConfigError::TooLarge {
            path: path.to_path_buf(),
            actual_bytes: metadata.len(),
            maximum_bytes: MAX_CONFIG_BYTES,
        });
    }

    let bytes = fs::read(path).map_err(|source| WebServerConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let instance: Value =
        serde_json::from_slice(&bytes).map_err(|source| WebServerConfigError::Json {
            path: path.to_path_buf(),
            source,
        })?;
    validate_schema(&instance)?;

    let config: WebServerAppConfig =
        serde_json::from_value(instance).map_err(|source| WebServerConfigError::Json {
            path: path.to_path_buf(),
            source,
        })?;
    validate_webserver_config(&config)?;

    let base_directory = path.parent().unwrap_or_else(|| Path::new("."));
    CompiledWebServerApp::compile(config, base_directory)
}

fn validate_schema(instance: &Value) -> Result<(), WebServerConfigError> {
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|error| WebServerConfigError::InvalidSchema(error.to_string()))?;
    let validator = jsonschema::draft202012::new(&schema)
        .map_err(|error| WebServerConfigError::InvalidSchema(error.to_string()))?;

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
        Err(WebServerConfigError::Validation { diagnostics })
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
