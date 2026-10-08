use axum::{
    Router,
    routing::{get, put},
};

use crate::shared::AppState;

use super::handlers;

/// Reads for the Ministry dashboard, no login.
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/dams", get(handlers::get_dams))
        .route("/dams/{slug}/history", get(handlers::get_dam_history))
}

/// Writes by the data jobs. Mounted under `/v1/ingest`, behind the service key.
pub fn ingest_routes() -> Router<AppState> {
    Router::new().route(
        "/dams/{slug}/readings/{day}",
        put(handlers::put_dam_reading),
    )
}
