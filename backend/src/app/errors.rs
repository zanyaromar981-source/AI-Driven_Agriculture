use crate::shared::{DomainError, JwtError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidInput,
    Authentication,
    Authorization,
    NotFound,
    Conflict,
    Persistence,
    UpstreamUnavailable,
    UpstreamRejected,
    UpstreamInvalidResponse,
    UpstreamFailure,
    Internal,
}

impl ErrorKind {
    pub const fn default_title(self) -> &'static str {
        match self {
            ErrorKind::InvalidInput => "Validation Error",
            ErrorKind::Authentication => "Unauthorized",
            ErrorKind::Authorization => "Forbidden",
            ErrorKind::NotFound => "Resource Not Found",
            ErrorKind::Conflict => "Conflict",
            ErrorKind::Persistence => "Repository Error",
            ErrorKind::UpstreamUnavailable => "Upstream Unavailable",
            ErrorKind::UpstreamRejected => "Upstream Rejected Request",
            ErrorKind::UpstreamInvalidResponse => "Invalid Upstream Response",
            ErrorKind::UpstreamFailure => "Integration Error",
            ErrorKind::Internal => "Internal Server Error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorInfo {
    pub kind: ErrorKind,
    pub title: &'static str,
    pub detail: String,
}

impl ErrorInfo {
    pub fn new(kind: ErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            title: kind.default_title(),
            detail: detail.into(),
        }
    }

    pub fn with_title(kind: ErrorKind, title: &'static str, detail: impl Into<String>) -> Self {
        Self {
            kind,
            title,
            detail: detail.into(),
        }
    }
}

pub trait ToErrorInfo {
    fn to_error_info(&self) -> ErrorInfo;
}

#[derive(thiserror::Error, Debug)]
pub enum IntegrationError {
    #[error("Upstream temporarily unavailable")]
    TemporarilyUnavailable,

    #[error("Upstream rejected the request")]
    Rejected,

    #[error("Upstream request failed: {0}")]
    RequestFailed(String),

    #[error("Conversion failed: {0}")]
    ConversionFailed(String),
}

impl ToErrorInfo for DomainError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            DomainError::InvalidValue(_) => {
                ErrorInfo::new(ErrorKind::InvalidInput, self.to_string())
            }
            DomainError::DuplicateValue(_) => ErrorInfo::new(ErrorKind::Conflict, self.to_string()),
        }
    }
}

impl ToErrorInfo for JwtError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            JwtError::SigningFailed => {
                ErrorInfo::new(ErrorKind::Internal, "The token could not be signed")
            }
            JwtError::MissingAuthorizationHeader
            | JwtError::InvalidFormat
            | JwtError::InvalidSubject
            | JwtError::TokenExpired
            | JwtError::InvalidIssuer
            | JwtError::InvalidAudience => {
                ErrorInfo::new(ErrorKind::Authentication, self.to_string())
            }
        }
    }
}

impl ToErrorInfo for IntegrationError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            IntegrationError::TemporarilyUnavailable => {
                ErrorInfo::new(ErrorKind::UpstreamUnavailable, self.to_string())
            }
            IntegrationError::Rejected => {
                ErrorInfo::new(ErrorKind::UpstreamRejected, self.to_string())
            }
            IntegrationError::RequestFailed(_) => {
                ErrorInfo::new(ErrorKind::UpstreamFailure, self.to_string())
            }
            IntegrationError::ConversionFailed(_) => {
                ErrorInfo::new(ErrorKind::UpstreamInvalidResponse, self.to_string())
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error(transparent)]
    IntegrationError(#[from] IntegrationError),

    #[error(transparent)]
    JwtError(#[from] JwtError),

    #[error("Forbidden")]
    Forbidden,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal server error")]
    InternalServerError,

    #[error("Not found")]
    NotFound,

    #[error("Database error: {0}")]
    FeatureError(String),

    #[error("Invalid value: {0}")]
    DomainError(#[from] DomainError),

    #[error("Missing value: {0}")]
    MissingValue(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::DatabaseError(err) => ErrorInfo::new(ErrorKind::Persistence, err.clone()),
            AppError::IntegrationError(err) => err.to_error_info(),
            AppError::JwtError(err) => err.to_error_info(),
            AppError::Forbidden => ErrorInfo::new(
                ErrorKind::Authorization,
                "You do not have permission to access this resource",
            ),
            AppError::Unauthorized(_) => ErrorInfo::new(
                ErrorKind::Authentication,
                "You are not authorized to access this resource",
            ),
            AppError::InternalServerError => {
                ErrorInfo::new(ErrorKind::Internal, "An unexpected error occurred")
            }
            AppError::NotFound => {
                ErrorInfo::new(ErrorKind::NotFound, "The requested resource was not found")
            }
            AppError::FeatureError(msg) => ErrorInfo::new(ErrorKind::Internal, msg.clone()),
            AppError::DomainError(err) => err.to_error_info(),
            AppError::MissingValue(detail) => ErrorInfo::new(ErrorKind::Internal, detail.clone()),
            AppError::InvalidInput(detail) => {
                ErrorInfo::new(ErrorKind::InvalidInput, detail.clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_database_failure_is_a_server_problem_not_the_callers() {
        let kind = AppError::DatabaseError("connection refused".to_string())
            .to_error_info()
            .kind;

        assert_eq!(kind, ErrorKind::Persistence);
    }

    #[test]
    fn a_duplicate_value_is_a_conflict_while_a_bad_value_is_invalid_input() {
        assert_eq!(
            DomainError::DuplicateValue("x".to_string())
                .to_error_info()
                .kind,
            ErrorKind::Conflict
        );
        assert_eq!(
            DomainError::InvalidValue("x".to_string())
                .to_error_info()
                .kind,
            ErrorKind::InvalidInput
        );
    }

    #[test]
    fn an_unreachable_upstream_is_told_apart_from_one_that_refused() {
        assert_eq!(
            IntegrationError::TemporarilyUnavailable
                .to_error_info()
                .kind,
            ErrorKind::UpstreamUnavailable
        );
        assert_eq!(
            IntegrationError::Rejected.to_error_info().kind,
            ErrorKind::UpstreamRejected,
            "a refusal is the upstream's business rule, not an outage"
        );
    }

    #[test]
    fn authorization_and_authentication_do_not_collapse_into_each_other() {
        assert_eq!(
            AppError::Forbidden.to_error_info().kind,
            ErrorKind::Authorization
        );
        assert_eq!(
            AppError::Unauthorized("token expired".to_string())
                .to_error_info()
                .kind,
            ErrorKind::Authentication
        );
    }

    #[test]
    fn an_unauthorized_reason_is_never_echoed_back() {
        let info =
            AppError::Unauthorized("signature mismatch for kid abc123".to_string()).to_error_info();

        assert!(
            !info.detail.contains("abc123"),
            "the reason a token failed must not help someone forge the next one"
        );
    }

    #[test]
    fn every_kind_has_a_title() {
        for kind in [
            ErrorKind::InvalidInput,
            ErrorKind::Authentication,
            ErrorKind::Authorization,
            ErrorKind::NotFound,
            ErrorKind::Conflict,
            ErrorKind::Persistence,
            ErrorKind::UpstreamUnavailable,
            ErrorKind::UpstreamRejected,
            ErrorKind::UpstreamInvalidResponse,
            ErrorKind::UpstreamFailure,
            ErrorKind::Internal,
        ] {
            assert!(!kind.default_title().is_empty(), "{kind:?} has no title");
        }
    }
}
