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
        .route("/dams", get(handlers::get_dams))
        .route("/dams/{slug}/history", get(handlers::get_dam_history))
}

/// Writes by the data jobs. Mounted under `/v1/ingest`, behind the service key.
pub fn ingest_routes() -> Router<AppState> {
    Router::new().route(
        "/dams/{slug}/readings/{day}",
        put(handlers::put_dam_reading),
    )
}

/// Reads and corrections by Ministry staff. Mounted under `/v1/dashboard`,
/// behind the `staff_auth` layer. Each method carries the one permission it
/// needs. The dams themselves are reference data and have no write here.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/dams",
            get(handlers::dashboard_get_dams).route_layer(require!(Resource::Dams, Action::Read)),
        )
        .route(
            "/dams/{slug}/readings",
            get(handlers::dashboard_get_dam_readings)
                .route_layer(require!(Resource::Dams, Action::Read))
                .merge(
                    post(handlers::dashboard_create_dam_reading)
                        .route_layer(require!(Resource::Dams, Action::Create)),
                ),
        )
        .route(
            "/dams/{slug}/readings/{day}",
            put(handlers::dashboard_update_dam_reading)
                .route_layer(require!(Resource::Dams, Action::Update))
                .merge(
                    delete(handlers::dashboard_delete_dam_reading)
                        .route_layer(require!(Resource::Dams, Action::Delete)),
                ),
        )
}
