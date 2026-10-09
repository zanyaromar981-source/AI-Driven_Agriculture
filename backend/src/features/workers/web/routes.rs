use axum::{
    Router,
    routing::{delete, get},
};

use crate::{
    app::{Action, Resource},
    require,
    shared::AppState,
};

use super::handlers;

/// Every route here needs the token, the list too: a card shows a phone
/// number, and phone numbers are shown only to signed-in people. None of
/// them may ever move to a public router.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/workers", get(handlers::get_workers))
        .route(
            "/workers/me",
            get(handlers::get_my_worker_card)
                .put(handlers::put_my_worker_card)
                .delete(handlers::delete_my_worker_card),
        )
}

/// Routes behind the `staff_auth` layer, for Ministry staff. A worker is an
/// ordinary farmer account, so the cards are guarded by the `farmers`
/// resource.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/workers",
            get(handlers::dashboard_get_workers)
                .route_layer(require!(Resource::Farmers, Action::Read)),
        )
        .route(
            "/workers/{id}",
            delete(handlers::dashboard_delete_worker)
                .route_layer(require!(Resource::Farmers, Action::Delete)),
        )
}
