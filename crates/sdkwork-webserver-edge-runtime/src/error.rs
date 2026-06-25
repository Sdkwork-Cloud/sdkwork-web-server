use thiserror::Error;

#[derive(Debug, Error)]
pub enum EdgeRuntimeError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("filesystem error: {0}")]
    Filesystem(String),
    #[error("nginx error: {0}")]
    Nginx(String),
}

pub type EdgeRuntimeResult<T> = Result<T, EdgeRuntimeError>;
