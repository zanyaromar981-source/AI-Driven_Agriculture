use axum::{
    Router,
    routing::{get, post},
};

use crate::shared::AppState;

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
