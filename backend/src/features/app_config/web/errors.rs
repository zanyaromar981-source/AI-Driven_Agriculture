use axum::response::{IntoResponse, Response};

use crate::{features::app_config::app::AppError, infra::http::HttpErrorResponse};

#[derive(Debug, thiserror::Error)]
pub enum WebError {
    #[error(transparent)]
    AppError(#[from] AppError),
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        match &self {
            WebError::AppError(err) => HttpErrorResponse::from_error(err).into_response(),
        }
    }
}
