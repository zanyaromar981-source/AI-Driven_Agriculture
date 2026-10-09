use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{delete, get, post, put},
};

use crate::{
    app::{Action, Resource},
    features::messages::domain::{Message, MessageText, Photo},
    require,
    shared::AppState,
};

use super::handlers;

/// Four photos at their largest plus room for the text, the other fields
/// and the form's own boundaries. Every other route keeps axum's 2 MB.
const SEND_BODY_LIMIT: usize =
    Message::MAX_PHOTOS * Photo::MAX_BYTES + MessageText::MAX_BYTES + 2 * 1024 * 1024;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/messages",
            post(handlers::send_message).layer(DefaultBodyLimit::max(SEND_BODY_LIMIT)),
        )
        .route("/messages/mine", get(handlers::get_my_messages))
        .route(
            "/messages/{id}/photos/{photo_id}",
            get(handlers::get_my_message_photo),
        )
}

/// Routes behind the `staff_auth` layer, for Ministry staff. Each method
/// carries the one permission it needs. They show every farmer's messages
/// with names and phones, so none of them may ever be merged into `routes()`.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/messages",
            get(handlers::dashboard_get_messages)
                .route_layer(require!(Resource::Messages, Action::Read)),
        )
        .route(
            "/messages/counts",
            get(handlers::dashboard_get_message_counts)
                .route_layer(require!(Resource::Messages, Action::Read)),
        )
        .route(
            "/messages/{id}",
            get(handlers::dashboard_get_message)
                .route_layer(require!(Resource::Messages, Action::Read))
                .merge(
                    put(handlers::dashboard_set_message_state)
                        .route_layer(require!(Resource::Messages, Action::Update)),
                )
                .merge(
                    delete(handlers::dashboard_delete_message)
                        .route_layer(require!(Resource::Messages, Action::Delete)),
                ),
        )
        .route(
            "/messages/{id}/reply",
            // A reply changes a message that exists, so it needs `update`,
            // not `create`.
            post(handlers::dashboard_reply_to_message)
                .route_layer(require!(Resource::Messages, Action::Update)),
        )
        .route(
            "/messages/{id}/photos/{photo_id}",
            get(handlers::dashboard_get_message_photo)
                .route_layer(require!(Resource::Messages, Action::Read)),
        )
}
