use axum::{Router, routing::get};

use crate::shared::AppState;

use super::handlers;

pub fn public_routes() -> Router<AppState> {
    Router::new().route("/versions", get(handlers::get_versions))
}

/// Behind the `staff_auth` layer and open to any signed-in staff member,
/// like `/me`: every dashboard screen needs it to know what to refresh.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new().route("/versions", get(handlers::get_dashboard_versions))
}
