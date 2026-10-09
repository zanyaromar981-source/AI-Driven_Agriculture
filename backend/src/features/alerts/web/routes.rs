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
    Router::new()
        .route("/farms/{id}/alerts", get(handlers::get_farm_alerts))
        .route("/alerts", get(handlers::get_my_alerts))
        .route("/alerts/{id}/done", post(handlers::mark_alert_done))
        .route("/devices", post(handlers::register_device))
        .route("/devices/{push_token}", delete(handlers::delete_device))
}

/// Used by the data jobs and, later, the push sender. Mounted under
/// `/ingest`, behind the service key. The only routes that ever answer a
/// push token.
pub fn ingest_routes() -> Router<AppState> {
    Router::new()
        .route("/farms/{id}/alerts/{key}", put(handlers::put_alert))
        .route("/alerts/unpushed", get(handlers::get_unpushed_alerts))
        .route("/alerts/{id}/pushed", post(handlers::mark_alert_pushed))
        .route(
            "/devices/{push_token}",
            delete(handlers::delete_dead_device),
        )
}

/// Used by staff to see and remove the stored alerts of any farm. Mounted
/// under `/dashboard`, behind the `staff_auth` layer; each method carries
/// the one permission it needs.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/farms/{id}/alerts",
            get(handlers::get_dashboard_farm_alerts)
                .route_layer(require!(Resource::Insights, Action::Read)),
        )
        .route(
            "/alerts/{id}",
            delete(handlers::delete_dashboard_alert)
                .route_layer(require!(Resource::Insights, Action::Delete)),
        )
}
