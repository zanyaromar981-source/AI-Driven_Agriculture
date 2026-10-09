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
    Router::new().nest(
        "/farms",
        Router::new()
            .route("/", get(handlers::get_farms).post(handlers::create_farm))
            .route(
                "/{id}",
                get(handlers::get_farm)
                    .put(handlers::edit_farm)
                    .delete(handlers::delete_farm),
            )
            .route("/{id}/status", get(handlers::get_farm_status))
            .route("/{id}/cells", put(handlers::repaint_farm_cells)),
    )
}

/// Routes behind the `staff_auth` layer, for Ministry staff. Each method
/// carries the one permission it needs. They reach every farmer's farms, so
/// none of them may ever be merged into `routes()`.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/farms",
            get(handlers::dashboard_get_farms)
                .route_layer(require!(Resource::Farms, Action::Read))
                .merge(
                    post(handlers::dashboard_create_farm)
                        .route_layer(require!(Resource::Farms, Action::Create)),
                ),
        )
        .route(
            "/farms/{id}",
            get(handlers::dashboard_get_farm)
                .route_layer(require!(Resource::Farms, Action::Read))
                .merge(
                    put(handlers::dashboard_rename_farm)
                        .route_layer(require!(Resource::Farms, Action::Update)),
                )
                .merge(
                    delete(handlers::dashboard_delete_farm)
                        .route_layer(require!(Resource::Farms, Action::Delete)),
                ),
        )
}
