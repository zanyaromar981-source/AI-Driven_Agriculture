use axum::{
    Router,
    routing::{get, put},
};

use crate::shared::AppState;

use super::handlers;

/// Reads for the Ministry dashboard, no login.
pub fn public_routes() -> Router<AppState> {
    Router::new().route("/water/plan", get(handlers::get_water_plan))
}

/// Writes by the data jobs. Mounted under `/v1/ingest`, behind the service key.
pub fn ingest_routes() -> Router<AppState> {
    Router::new().route(
        "/water/plan/{season}/zones/{zone_slug}",
        put(handlers::put_water_plan_entry).delete(handlers::delete_water_plan_entry),
    )
}
