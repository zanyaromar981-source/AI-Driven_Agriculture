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

/// Reads and corrections by Ministry staff. Mounted under `/v1/dashboard`,
/// behind the `staff_auth` layer. Each method carries the one permission it
/// needs.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/water/seasons",
            get(handlers::dashboard_get_water_seasons)
                .route_layer(require!(Resource::Water, Action::Read)),
        )
        .route(
            "/water/plan/{season}/entries",
            get(handlers::dashboard_get_water_plan_entries)
                .route_layer(require!(Resource::Water, Action::Read))
                .merge(
                    post(handlers::dashboard_create_water_plan_entry)
                        .route_layer(require!(Resource::Water, Action::Create)),
                ),
        )
        .route(
            "/water/plan/{season}/entries/{zone_slug}",
            put(handlers::dashboard_update_water_plan_entry)
                .route_layer(require!(Resource::Water, Action::Update))
                .merge(
                    delete(handlers::dashboard_delete_water_plan_entry)
                        .route_layer(require!(Resource::Water, Action::Delete)),
                ),
        )
}
