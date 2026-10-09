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
    Router::new()
        .route("/outlooks", get(handlers::get_season_outlook))
        .route(
            "/outlooks/zones/{zone_slug}",
            get(handlers::get_zone_outlook),
        )
}

/// Writes by the data jobs. Mounted under `/v1/ingest`, behind the service key.
pub fn ingest_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/outlooks/{season}/{issued}/zones/{zone_slug}",
            put(handlers::put_zone_outlook),
        )
        .route(
            "/outlooks/{season}/{issued}/run",
            put(handlers::put_outlook_run),
        )
}

/// Reads and corrections by Ministry staff. Mounted under `/v1/dashboard`,
/// behind the `staff_auth` layer. Each method carries the one permission it
/// needs.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/outlooks",
            get(handlers::dashboard_get_outlooks)
                .route_layer(require!(Resource::Outlooks, Action::Read))
                .merge(
                    post(handlers::dashboard_create_outlook)
                        .route_layer(require!(Resource::Outlooks, Action::Create)),
                ),
        )
        .route(
            "/outlooks/{season}/{issued}/zones/{zone_slug}",
            put(handlers::dashboard_update_outlook)
                .route_layer(require!(Resource::Outlooks, Action::Update))
                .merge(
                    delete(handlers::dashboard_delete_outlook)
                        .route_layer(require!(Resource::Outlooks, Action::Delete)),
                ),
        )
        .route(
            "/outlook-runs",
            get(handlers::dashboard_get_outlook_runs)
                .route_layer(require!(Resource::Outlooks, Action::Read))
                .merge(
                    post(handlers::dashboard_create_outlook_run)
                        .route_layer(require!(Resource::Outlooks, Action::Create)),
                ),
        )
        .route(
            "/outlook-runs/{season}/{issued}",
            put(handlers::dashboard_update_outlook_run)
                .route_layer(require!(Resource::Outlooks, Action::Update))
                .merge(
                    delete(handlers::dashboard_delete_outlook_run)
                        .route_layer(require!(Resource::Outlooks, Action::Delete)),
                ),
        )
}
