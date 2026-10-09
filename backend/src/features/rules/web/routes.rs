use axum::{
    Router,
    routing::{get, post, put},
};

use crate::{
    app::{Action, Resource},
    require,
    shared::AppState,
};

use super::handlers;

/// The read the data jobs make once per run. Mounted under `/v1/ingest`,
/// behind the service key.
pub fn ingest_routes() -> Router<AppState> {
    Router::new().route("/rules", get(handlers::get_rule_values))
}

/// Reads and changes by Ministry staff. Mounted under `/v1/dashboard`,
/// behind the `staff_auth` layer. Each method carries the one permission it
/// needs. There is no create and no delete: a rule exists only by migration,
/// because each one is read by code. A reset is a change of the value, so it
/// needs the same permission as one.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/rules",
            get(handlers::dashboard_get_rules).route_layer(require!(Resource::Rules, Action::Read)),
        )
        .route(
            "/rules/{code}",
            put(handlers::dashboard_change_rule)
                .route_layer(require!(Resource::Rules, Action::Update)),
        )
        .route(
            "/rules/{code}/history",
            get(handlers::dashboard_get_rule_history)
                .route_layer(require!(Resource::Rules, Action::Read)),
        )
        .route(
            "/rules/{code}/reset",
            post(handlers::dashboard_reset_rule)
                .route_layer(require!(Resource::Rules, Action::Update)),
        )
}
