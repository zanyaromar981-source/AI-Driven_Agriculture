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

/// What the Ministry dashboard reads. No login.
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/briefs/latest", get(handlers::get_latest_brief))
        .route("/briefs", get(handlers::get_briefs))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/farms/{id}/brief", get(handlers::get_farm_brief))
}

/// What the nightly job writes. Mounted under `/ingest`, behind the service
/// key.
pub fn ingest_routes() -> Router<AppState> {
    Router::new()
        .route("/briefs/farm-zones", put(handlers::put_farm_zones))
        .route(
            "/briefs/{day}/{scope}",
            put(handlers::put_brief).delete(handlers::delete_brief),
        )
}

/// Used by staff to correct or remove a brief. Mounted under `/dashboard`,
/// behind the `staff_auth` layer; each method carries the one permission it
/// needs. There is no create: only the nightly job writes new briefs.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/briefs",
            get(handlers::get_dashboard_briefs)
                .route_layer(require!(Resource::Briefs, Action::Read)),
        )
        .route(
            "/briefs/{day}/{scope}",
            put(handlers::update_dashboard_brief)
                .route_layer(require!(Resource::Briefs, Action::Update))
                .merge(
                    delete(handlers::delete_dashboard_brief)
                        .route_layer(require!(Resource::Briefs, Action::Delete)),
                ),
        )
}
