use axum::{
    Router,
    routing::{get, put},
};

use crate::{
    app::{Action, Resource},
    require,
    shared::AppState,
};

use super::handlers;

/// No login: the app reads this at start, before anyone has signed in. It
/// also sits outside the version check, so an app too old for everything
/// else can still learn here that it must update.
pub fn public_routes() -> Router<AppState> {
    Router::new().route("/app/config", get(handlers::get_app_config))
}

/// Routes behind the `staff_auth` layer, for Ministry staff. Each method
/// carries the one permission it needs.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/app/config",
            get(handlers::dashboard_get_app_config)
                .route_layer(require!(Resource::App, Action::Read))
                .merge(
                    put(handlers::dashboard_update_app_config)
                        .route_layer(require!(Resource::App, Action::Update)),
                ),
        )
        .route(
            "/app/versions",
            get(handlers::dashboard_get_app_versions)
                .route_layer(require!(Resource::App, Action::Read)),
        )
}
