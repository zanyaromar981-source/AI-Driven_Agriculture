use axum::{
    Router,
    routing::{get, put},
};

use crate::shared::AppState;

use super::handlers;

pub fn routes() -> Router<AppState> {
    Router::new().route("/farms/{id}/insights", get(handlers::get_farm_insights))
}

/// Used by the data jobs. Mounted under `/ingest`, behind the service key.
pub fn ingest_routes() -> Router<AppState> {
    Router::new()
        .route("/farms", get(handlers::get_farm_coverage))
        .route(
            "/farms/{id}/insights/{topic}",
            put(handlers::put_farm_insight),
        )
}
