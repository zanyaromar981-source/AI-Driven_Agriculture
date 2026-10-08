use axum::{
    extract::rejection::{JsonRejection, PathRejection, QueryRejection},
    response::{IntoResponse, Response},
};

use crate::{
    app::AppError as GlobalAppError, features::staff::app::AppError, infra::http::HttpErrorResponse,
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
