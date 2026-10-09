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

/// Read by the Ministry dashboard, which has no login.
pub fn public_routes() -> Router<AppState> {
    Router::new().route("/fires", get(handlers::get_fires))
}

/// Written by the fire detection job. Mounted under `/ingest`, behind the
/// service key.
pub fn ingest_routes() -> Router<AppState> {
    Router::new().route("/fires/{external_id}", put(handlers::put_fire))
}

/// Used by staff to see and correct the stored fires. Mounted under
/// `/dashboard`, behind the `staff_auth` layer; each method carries the one
/// permission it needs.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/fires",
            get(handlers::get_dashboard_fires)
                .route_layer(require!(Resource::Fires, Action::Read))
                .merge(
                    post(handlers::create_dashboard_fire)
                        .route_layer(require!(Resource::Fires, Action::Create)),
                ),
        )
        .route(
            "/fires/{id}",
            get(handlers::get_dashboard_fire)
                .route_layer(require!(Resource::Fires, Action::Read))
                .merge(
                    put(handlers::update_dashboard_fire)
                        .route_layer(require!(Resource::Fires, Action::Update)),
                )
                .merge(
                    delete(handlers::delete_dashboard_fire)
                        .route_layer(require!(Resource::Fires, Action::Delete)),
                ),
        )
}
