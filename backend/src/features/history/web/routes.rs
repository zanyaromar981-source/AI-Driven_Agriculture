use axum::{
    Router,
    routing::{delete, get, put},
};

use crate::{
    app::{Action, Resource},
    require,
    shared::AppState,
};

use super::handlers;

pub fn routes() -> Router<AppState> {
    Router::new().route("/farms/{id}/history", get(handlers::get_farm_history))
}

/// Used by the history job. Mounted under `/ingest`, behind the service key.
/// `/farms/history/coverage` cannot be taken for a farm whose id is
/// "history": the router prefers a fixed segment to `{id}`, and the push
/// route is one segment longer and answers only PUT.
pub fn ingest_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/farms/history/coverage",
            get(handlers::get_history_coverage),
        )
        .route(
            "/farms/{id}/history/{metric}",
            put(handlers::put_farm_history),
        )
}

/// Used by staff to see the stored history of any farm and to clear a bad
/// series. Mounted under `/dashboard`, behind the `staff_auth` layer; each
/// method carries the one permission it needs.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/farms/{id}/history",
            get(handlers::get_dashboard_farm_history)
                .route_layer(require!(Resource::Insights, Action::Read)),
        )
        .route(
            "/farms/{id}/history/{metric}",
            delete(handlers::delete_dashboard_farm_history)
                .route_layer(require!(Resource::Insights, Action::Delete)),
        )
}
