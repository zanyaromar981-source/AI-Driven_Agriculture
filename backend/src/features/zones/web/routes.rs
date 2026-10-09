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

/// What the Ministry dashboard reads. No login.
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/region/overview", get(handlers::get_region_overview))
        .route("/region/compare", get(handlers::get_region_comparison))
        .route("/zones/{slug}", get(handlers::get_zone))
}

/// What the data jobs write. Mounted under `/ingest`, behind the service key.
pub fn ingest_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/zones/{slug}/readings/{month}",
            put(handlers::put_zone_reading),
        )
        .route(
            "/zones/{slug}/sub-zones/{sub_slug}/readings/{month}",
            put(handlers::put_sub_zone_reading),
        )
}

/// What staff see and correct on the dashboard. Mounted under `/dashboard`,
/// behind the `staff_auth` layer; each method carries the one permission it
/// needs. Zones and sub-zones are reference data, so they are only read.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/zones",
            get(handlers::get_dashboard_zones).route_layer(require!(Resource::Zones, Action::Read)),
        )
        .route(
            "/zones/{slug}/readings",
            get(handlers::get_dashboard_zone_readings)
                .route_layer(require!(Resource::Zones, Action::Read))
                .merge(
                    post(handlers::create_dashboard_zone_reading)
                        .route_layer(require!(Resource::Zones, Action::Create)),
                ),
        )
        .route(
            "/zones/{slug}/readings/{month}",
            put(handlers::update_dashboard_zone_reading)
                .route_layer(require!(Resource::Zones, Action::Update))
                .merge(
                    delete(handlers::delete_dashboard_zone_reading)
                        .route_layer(require!(Resource::Zones, Action::Delete)),
                ),
        )
        .route(
            "/zones/{slug}/sub-zones/{sub_slug}/readings",
            get(handlers::get_dashboard_sub_zone_readings)
                .route_layer(require!(Resource::Zones, Action::Read))
                .merge(
                    post(handlers::create_dashboard_sub_zone_reading)
                        .route_layer(require!(Resource::Zones, Action::Create)),
                ),
        )
        .route(
            "/zones/{slug}/sub-zones/{sub_slug}/readings/{month}",
            put(handlers::update_dashboard_sub_zone_reading)
                .route_layer(require!(Resource::Zones, Action::Update))
                .merge(
                    delete(handlers::delete_dashboard_sub_zone_reading)
                        .route_layer(require!(Resource::Zones, Action::Delete)),
                ),
        )
}
