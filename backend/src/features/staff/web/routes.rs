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

/// The one dashboard route that needs no token: it is how staff get one.
pub fn dashboard_public_routes() -> Router<AppState> {
    Router::new().route("/auth/login", post(handlers::sign_in))
}

/// Routes behind the `staff_auth` layer. Each method carries the one
/// permission it needs; `/me` and `/permissions` are open to anyone signed
/// in, because every dashboard screen needs them.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(handlers::get_me).put(handlers::update_me))
        .route("/permissions", get(handlers::get_permission_catalogue))
        .route(
            "/roles",
            get(handlers::get_roles)
                .route_layer(require!(Resource::Roles, Action::Read))
                .merge(
                    post(handlers::create_role)
                        .route_layer(require!(Resource::Roles, Action::Create)),
                ),
        )
        .route(
            "/roles/{id}",
            get(handlers::get_role)
                .route_layer(require!(Resource::Roles, Action::Read))
                .merge(
                    put(handlers::update_role)
                        .route_layer(require!(Resource::Roles, Action::Update)),
                )
                .merge(
                    delete(handlers::delete_role)
                        .route_layer(require!(Resource::Roles, Action::Delete)),
                ),
        )
        .route(
            "/staff",
            get(handlers::get_staff_list)
                .route_layer(require!(Resource::Staff, Action::Read))
                .merge(
                    post(handlers::create_staff)
                        .route_layer(require!(Resource::Staff, Action::Create)),
                ),
        )
        .route(
            "/staff/{id}",
            get(handlers::get_staff)
                .route_layer(require!(Resource::Staff, Action::Read))
                .merge(
                    put(handlers::update_staff)
                        .route_layer(require!(Resource::Staff, Action::Update)),
                )
                .merge(
                    delete(handlers::delete_staff)
                        .route_layer(require!(Resource::Staff, Action::Delete)),
                ),
        )
}
