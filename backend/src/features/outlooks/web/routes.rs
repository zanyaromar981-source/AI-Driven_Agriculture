use axum::{
    Router,
    routing::{get, put},
};

use crate::shared::AppState;

use super::handlers;

/// Reads for the Ministry dashboard, no login.
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/outlooks", get(handlers::get_season_outlook))
        .route(
            "/outlooks/zones/{zone_slug}",
            get(handlers::get_zone_outlook),
        )
}

/// Writes by the data jobs. Mounted under `/v1/ingest`, behind the service key.
pub fn ingest_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/outlooks/{season}/{issued}/zones/{zone_slug}",
            put(handlers::put_zone_outlook),
        )
        .route(
            "/outlooks/{season}/{issued}/run",
            put(handlers::put_outlook_run),
        )
}
