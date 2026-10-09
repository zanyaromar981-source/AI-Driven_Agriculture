use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::{
    app::{Action, Resource},
    require,
    shared::AppState,
};

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

/// Used by staff to see and correct the stored readings of any farm.
/// Mounted under `/dashboard`, behind the `staff_auth` layer; each method
/// carries the one permission it needs.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/farms/{id}/insights",
            get(handlers::get_dashboard_farm_insights)
                .route_layer(require!(Resource::Insights, Action::Read))
                .merge(
                    post(handlers::create_dashboard_farm_insight)
                        .route_layer(require!(Resource::Insights, Action::Create)),
                ),
        )
        .route(
            "/farms/{id}/insights/{topic}",
            put(handlers::update_dashboard_farm_insight)
                .route_layer(require!(Resource::Insights, Action::Update))
                .merge(
                    delete(handlers::delete_dashboard_farm_insight)
                        .route_layer(require!(Resource::Insights, Action::Delete)),
                ),
        )
}
