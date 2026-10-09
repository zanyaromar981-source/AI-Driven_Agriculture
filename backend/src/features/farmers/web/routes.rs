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

/// Routes that need no token: they are how a farmer gets one.
pub fn public_routes() -> Router<AppState> {
    Router::new().nest(
        "/auth/otp",
        Router::new()
            .route("/send", post(handlers::send_sign_in_code))
            .route("/verify", post(handlers::verify_sign_in_code)),
    )
}

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/me",
        get(handlers::get_profile).put(handlers::update_profile),
    )
}

/// Routes behind the `staff_auth` layer, for Ministry staff. Each method
/// carries the one permission it needs. They show farmers' phones, so none
/// of them may ever be merged into `routes()` or `public_routes()`.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/farmers",
            get(handlers::dashboard_get_farmers)
                .route_layer(require!(Resource::Farmers, Action::Read))
                .merge(
                    post(handlers::dashboard_create_farmer)
                        .route_layer(require!(Resource::Farmers, Action::Create)),
                ),
        )
        .route(
            "/farmers/{id}",
            get(handlers::dashboard_get_farmer)
                .route_layer(require!(Resource::Farmers, Action::Read))
                .merge(
                    put(handlers::dashboard_update_farmer)
                        .route_layer(require!(Resource::Farmers, Action::Update)),
                )
                .merge(
                    delete(handlers::dashboard_delete_farmer)
                        .route_layer(require!(Resource::Farmers, Action::Delete)),
                ),
        )
        // Issuing a letter stores a record but changes nothing about the
        // farmer, and anyone who may read a farmer may print their letter:
        // it needs `read`, not `create`.
        .route(
            "/farmers/{id}/letters",
            post(handlers::dashboard_issue_letter)
                .route_layer(require!(Resource::Farmers, Action::Read)),
        )
        .route(
            "/letters/{number}",
            get(handlers::dashboard_get_letter)
                .route_layer(require!(Resource::Farmers, Action::Read)),
        )
}
