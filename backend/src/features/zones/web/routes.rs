use axum::{
    Router,
    routing::{get, put},
};

use crate::shared::AppState;

use super::handlers;

/// What the Ministry dashboard reads. No login.
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/region/overview", get(handlers::get_region_overview))
        .route("/region/compare", get(handlers::get_region_comparison))
        .route("/zones/{slug}", get(handlers::get_zone))
}

/// What the data jobs write. Mounted under `/ingest`, behind the service key.
pub fn ingest_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/zones/{slug}/readings/{month}",
            put(handlers::put_zone_reading),
        )
        .route(
            "/zones/{slug}/sub-zones/{sub_slug}/readings/{month}",
            put(handlers::put_sub_zone_reading),
        )
}
