use axum::{Router, extract::DefaultBodyLimit, routing::post};

use crate::{
    features::doctor::domain::{Enquiry, Photo, Question},
    shared::AppState,
};

use super::handlers;

/// Six photos at their largest plus room for the question, the other fields
/// and the form's own boundaries. Every other route keeps axum's 2 MB.
const ASK_BODY_LIMIT: usize =
    Enquiry::MAX_PHOTOS * Photo::MAX_BYTES + Question::MAX_BYTES + 2 * 1024 * 1024;

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/farms/{id}/ask",
        post(handlers::ask_doctor).layer(DefaultBodyLimit::max(ASK_BODY_LIMIT)),
    )
}
