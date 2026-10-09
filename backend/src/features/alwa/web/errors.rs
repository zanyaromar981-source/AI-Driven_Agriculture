use axum::{
    extract::rejection::{JsonRejection, PathRejection, QueryRejection},
    response::{IntoResponse, Response},
};

use crate::{
    app::AppError as GlobalAppError,
    features::alwa::{app::AppError, domain::AlwaError},
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
}

impl WebError {
    pub fn not_found() -> Self {
        Self::AppError(GlobalAppError::NotFound.into())
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        match &self {
            // The one rule failure that says which value of the body broke
            // it, in the shape a failed body validation has.
            WebError::AppError(AppError::Alwa(AlwaError::InvalidField { field, detail })) => {
                HttpErrorResponse::invalid_field(*field, detail.clone()).into_response()
            }
            WebError::AppError(err) => HttpErrorResponse::from_error(err).into_response(),
            WebError::JsonRejection(err) => {
                HttpErrorResponse::bad_request(err.to_string()).into_response()
            }
            WebError::PathRejection(err) => {
                HttpErrorResponse::bad_request(err.to_string()).into_response()
            }
            WebError::QueryRejection(err) => {
                HttpErrorResponse::bad_request(err.to_string()).into_response()
            }
        }
    }
}
