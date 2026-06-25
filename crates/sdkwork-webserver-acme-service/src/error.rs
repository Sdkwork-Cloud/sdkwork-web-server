use thiserror::Error;

#[derive(Debug, Error)]
pub enum AcmeServiceError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("acme provider error: {0}")]
    Provider(String),
    #[error("encryption error: {0}")]
    Encryption(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl AcmeServiceError {
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn provider(message: impl Into<String>) -> Self {
        Self::Provider(message.into())
    }
}

pub type AcmeServiceResult<T> = Result<T, AcmeServiceError>;
