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

/// What the app and the public page read without a login: the crops that
/// are switched on, with their names and colours.
pub fn public_routes() -> Router<AppState> {
    Router::new().route("/crops", get(handlers::get_crops))
}

/// What Ministry staff do on the dashboard. Mounted under `/v1/dashboard`,
/// behind the `staff_auth` layer; each method carries the one permission it
/// needs.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/crops",
            get(handlers::dashboard_get_crops)
                .route_layer(require!(Resource::Crops, Action::Read))
                .merge(
                    post(handlers::dashboard_create_crop)
                        .route_layer(require!(Resource::Crops, Action::Create)),
                ),
        )
        .route(
            "/crops/{code}",
            put(handlers::dashboard_update_crop)
                .route_layer(require!(Resource::Crops, Action::Update))
                .merge(
                    delete(handlers::dashboard_delete_crop)
                        .route_layer(require!(Resource::Crops, Action::Delete)),
                ),
        )
}
