use thiserror::Error;

use crate::ConfigDiagnostic;

#[derive(Debug, Error)]
pub enum WebsiteRuntimeDescriptorError {
    #[error("website runtime descriptor is {actual_bytes} bytes; maximum is {maximum_bytes}")]
    TooLarge {
        actual_bytes: usize,
        maximum_bytes: usize,
    },

    #[error("website runtime descriptor is not valid JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("embedded website runtime descriptor JSON Schema is invalid: {0}")]
    InvalidSchema(String),

    #[error(
        "website runtime descriptor hash mismatch: expected {expected}, calculated {calculated}"
    )]
    HashMismatch {
        expected: String,
        calculated: String,
    },

    #[error("website runtime descriptor failed validation")]
    Validation { diagnostics: Vec<ConfigDiagnostic> },
}

impl WebsiteRuntimeDescriptorError {
    pub fn diagnostics(&self) -> &[ConfigDiagnostic] {
        match self {
            Self::Validation { diagnostics } => diagnostics,
            _ => &[],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum WebsiteRouteSelectionError {
    #[error("request host is invalid")]
    InvalidHost,
    #[error("request path is invalid")]
    InvalidPath,
    #[error("request path is denied by the compiled website security policy")]
    DeniedPath,
}
