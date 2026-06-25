//! Web service error model aligned with OpenAPI problem responses.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebServiceErrorKind {
    NotFound,
    Conflict,
    Validation,
    Forbidden,
    DatabaseUnavailable,
    Internal,
}

#[derive(Debug, thiserror::Error)]
pub enum WebServiceError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("forbidden")]
    Forbidden,
    #[error("database unavailable")]
    DatabaseUnavailable,
    #[error("internal error: {0}")]
    Internal(String),
}

impl WebServiceError {
    pub fn kind(&self) -> WebServiceErrorKind {
        match self {
            Self::NotFound(_) => WebServiceErrorKind::NotFound,
            Self::Conflict(_) => WebServiceErrorKind::Conflict,
            Self::Validation(_) => WebServiceErrorKind::Validation,
            Self::Forbidden => WebServiceErrorKind::Forbidden,
            Self::DatabaseUnavailable => WebServiceErrorKind::DatabaseUnavailable,
            Self::Internal(_) => WebServiceErrorKind::Internal,
        }
    }

    pub fn not_found(detail: impl Into<String>) -> Self {
        Self::NotFound(detail.into())
    }

    pub fn conflict(detail: impl Into<String>) -> Self {
        Self::Conflict(detail.into())
    }

    pub fn validation(detail: impl Into<String>) -> Self {
        Self::Validation(detail.into())
    }
}

pub type WebServiceResult<T> = Result<T, WebServiceError>;
