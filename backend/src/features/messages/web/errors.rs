use axum::{
    extract::{
        multipart::{MultipartError, MultipartRejection},
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    app::AppError as GlobalAppError,
    features::messages::{
        app::AppError,
        domain::{MessageError, Photo},
    },
    infra::http::HttpErrorResponse,
};

#[derive(Debug, thiserror::Error)]
pub enum WebError {
    #[error(transparent)]
    AppError(#[from] AppError),
    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),
    #[error(transparent)]
    PathRejection(#[from] PathRejection),
    #[error(transparent)]
    QueryRejection(#[from] QueryRejection),
    #[error(transparent)]
    MultipartRejection(#[from] MultipartRejection),
    #[error(transparent)]
    Multipart(#[from] MultipartError),
    /// A form field that cannot be read, such as text that is not UTF-8.
    #[error("{0}")]
    BadField(String),
}

impl WebError {
    pub fn not_found() -> Self {
        Self::AppError(GlobalAppError::NotFound.into())
    }
}

impl From<MessageError> for WebError {
    fn from(error: MessageError) -> Self {
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
                HttpErrorResponse::from_error(&AppError::from(MessageError::PhotoTooLarge(
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
            WebError::JsonRejection(err) => {
                HttpErrorResponse::bad_request(err.to_string()).into_response()
            }
            WebError::PathRejection(err) => {
                HttpErrorResponse::bad_request(err.to_string()).into_response()
            }
            WebError::QueryRejection(err) => {
                HttpErrorResponse::bad_request(err.to_string()).into_response()
            }
            WebError::BadField(detail) => {
                HttpErrorResponse::bad_request(detail.clone()).into_response()
            }
        }
    }
}
