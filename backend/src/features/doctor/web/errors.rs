use axum::{
    extract::{
        multipart::{MultipartError, MultipartRejection},
        rejection::PathRejection,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    app::AppError as GlobalAppError,
    features::doctor::{
        app::AppError,
        domain::{DoctorError, Photo},
    },
    infra::http::HttpErrorResponse,
};

#[derive(Debug, thiserror::Error)]
pub enum WebError {
    #[error(transparent)]
    AppError(#[from] AppError),
    #[error(transparent)]
    PathRejection(#[from] PathRejection),
    #[error(transparent)]
    MultipartRejection(#[from] MultipartRejection),
    #[error(transparent)]
    Multipart(#[from] MultipartError),
    /// A form field that cannot be read, such as a cell that is not JSON.
    #[error("{0}")]
    BadField(String),
}

impl WebError {
    pub fn not_found() -> Self {
        Self::AppError(GlobalAppError::NotFound.into())
    }
}

impl From<DoctorError> for WebError {
    fn from(error: DoctorError) -> Self {
        Self::AppError(error.into())
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        match &self {
            WebError::AppError(err) => HttpErrorResponse::from_error(err).into_response(),
            // Only photos can make a form this large: the other fields are
            // capped well below the route's limit.
            WebError::Multipart(err) if err.status() == StatusCode::PAYLOAD_TOO_LARGE => {
                HttpErrorResponse::from_error(&AppError::from(DoctorError::PhotoTooLarge(
                    Photo::MAX_MB,
                )))
                .into_response()
            }
            WebError::Multipart(err) => {
                HttpErrorResponse::bad_request(err.body_text()).into_response()
            }
            WebError::MultipartRejection(err) => {
                HttpErrorResponse::bad_request(err.body_text()).into_response()
            }
            WebError::PathRejection(err) => {
                HttpErrorResponse::bad_request(err.to_string()).into_response()
            }
            WebError::BadField(detail) => {
                HttpErrorResponse::bad_request(detail.clone()).into_response()
            }
        }
    }
}
