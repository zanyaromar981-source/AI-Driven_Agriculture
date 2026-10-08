use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::app::{AppError, ErrorInfo, ErrorKind, ToErrorInfo};

/// The body of every error answer: `{"error": "<code>", ...}`. The app acts
/// on `error`; `detail` is English text for people reading logs.
#[derive(Serialize, Debug, ToSchema)]
pub struct ErrorBody {
    pub error: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_s: Option<u64>,
}

#[derive(Debug)]
pub struct HttpErrorResponse {
    status: StatusCode,
    code: &'static str,
    detail: String,
    field: Option<String>,
    retry_after_s: Option<u64>,
}

impl HttpErrorResponse {
    pub fn from_error<T: ToErrorInfo + ?Sized>(error: &T) -> Self {
        error.to_error_info().into()
    }

    pub fn bad_request(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "bad_request",
            detail: detail.into(),
            field: None,
            retry_after_s: None,
        }
    }

    pub fn invalid_field(field: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: ErrorKind::InvalidInput.default_code(),
            detail: detail.into(),
            field: Some(field.into()),
            retry_after_s: None,
        }
    }
}

/// Sent instead of the real detail whenever the status is a `5xx`. A server-side
/// failure is never the caller's to act on, and the detail behind one carries our
/// internals: SQL text, table names, upstream error bodies.
const SERVER_ERROR_DETAIL: &str = "An unexpected error occurred";

impl From<ErrorInfo> for HttpErrorResponse {
    fn from(info: ErrorInfo) -> Self {
        let status = status_for_error_kind(info.kind);

        let detail = if status.is_server_error() {
            SERVER_ERROR_DETAIL.to_string()
        } else {
            info.detail
        };

        Self {
            status,
            code: info.code,
            detail,
            field: None,
            retry_after_s: info.retry_after_s,
        }
    }
}

impl IntoResponse for HttpErrorResponse {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.code.to_string(),
                detail: self.detail,
                field: self.field,
                retry_after_s: self.retry_after_s,
            }),
        )
            .into_response()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        HttpErrorResponse::from_error(&self).into_response()
    }
}

fn status_for_error_kind(kind: ErrorKind) -> StatusCode {
    match kind {
        ErrorKind::InvalidInput => StatusCode::UNPROCESSABLE_ENTITY,
        ErrorKind::Authentication => StatusCode::UNAUTHORIZED,
        ErrorKind::Authorization => StatusCode::FORBIDDEN,
        ErrorKind::NotFound => StatusCode::NOT_FOUND,
        ErrorKind::Conflict => StatusCode::CONFLICT,
        ErrorKind::RateLimited => StatusCode::TOO_MANY_REQUESTS,
        ErrorKind::Persistence => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorKind::UpstreamUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        // The remote dependency understood the request but declined the
        // operation for its own business rules; this is not client input
        // validation in our API.
        ErrorKind::UpstreamRejected => StatusCode::FAILED_DEPENDENCY,
        ErrorKind::UpstreamInvalidResponse | ErrorKind::UpstreamFailure => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
        ErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_local_error_kinds_to_client_and_server_statuses() {
        assert_eq!(
            status_for_error_kind(ErrorKind::InvalidInput),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            status_for_error_kind(ErrorKind::Conflict),
            StatusCode::CONFLICT
        );
        assert_eq!(
            status_for_error_kind(ErrorKind::NotFound),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status_for_error_kind(ErrorKind::RateLimited),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            status_for_error_kind(ErrorKind::Persistence),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn maps_upstream_logical_rejection_to_failed_dependency() {
        assert_eq!(
            status_for_error_kind(ErrorKind::UpstreamRejected),
            StatusCode::FAILED_DEPENDENCY
        );
        assert_eq!(
            status_for_error_kind(ErrorKind::UpstreamFailure),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            status_for_error_kind(ErrorKind::UpstreamUnavailable),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[test]
    fn a_database_failure_never_reaches_the_caller_verbatim() {
        let leaked = "error returned from database: relation \"farms\" does not exist";

        let response = HttpErrorResponse::from(ErrorInfo::new(ErrorKind::Persistence, leaked));

        assert_eq!(response.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(response.detail, SERVER_ERROR_DETAIL);
        assert!(
            !response.detail.contains("relation"),
            "schema names must not travel to the client"
        );
    }

    #[test]
    fn every_server_error_kind_is_scrubbed() {
        for kind in [
            ErrorKind::Persistence,
            ErrorKind::Internal,
            ErrorKind::UpstreamFailure,
            ErrorKind::UpstreamInvalidResponse,
            ErrorKind::UpstreamUnavailable,
        ] {
            let response = HttpErrorResponse::from(ErrorInfo::new(kind, "internal detail"));

            assert_eq!(
                response.detail, SERVER_ERROR_DETAIL,
                "{kind:?} leaked its detail"
            );
        }
    }

    #[test]
    fn a_client_error_keeps_the_detail_it_needs_to_act_on() {
        let response = HttpErrorResponse::from(ErrorInfo::new(
            ErrorKind::InvalidInput,
            "Farm name must not be empty",
        ));

        assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            response.detail, "Farm name must not be empty",
            "a 4xx detail describes the caller's own input, so it must survive"
        );
    }

    #[test]
    fn the_code_still_says_which_failure_it_was() {
        let specific = HttpErrorResponse::from(ErrorInfo::with_code(
            ErrorKind::InvalidInput,
            "bad_polygon",
            "The farm outline crosses itself",
        ));
        let general = HttpErrorResponse::from(ErrorInfo::new(ErrorKind::Persistence, "raw"));

        assert_eq!(specific.code, "bad_polygon");
        assert_eq!(general.code, "server_error");
    }

    #[test]
    fn a_rate_limit_tells_the_caller_how_long_to_wait() {
        let response = HttpErrorResponse::from(
            ErrorInfo::new(ErrorKind::RateLimited, "Too many codes requested").retry_after(42),
        );

        assert_eq!(response.status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.retry_after_s, Some(42));
    }
}
