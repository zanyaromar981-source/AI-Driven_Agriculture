use axum::{
    Router,
    routing::{get, put},
};

use crate::shared::AppState;

use super::handlers;

/// Read by the Ministry dashboard, which has no login.
pub fn public_routes() -> Router<AppState> {
    Router::new().route("/fires", get(handlers::get_fires))
}

/// Written by the fire detection job. Mounted under `/ingest`, behind the
/// service key.
pub fn ingest_routes() -> Router<AppState> {
    Router::new().route("/fires/{external_id}", put(handlers::put_fire))
}
