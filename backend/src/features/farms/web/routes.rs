use axum::{
    Router,
    routing::{delete, get, put},
};

use crate::shared::AppState;

use super::handlers;

pub fn routes() -> Router<AppState> {
    Router::new().nest(
        "/farms",
        Router::new()
            .route("/", get(handlers::get_farms).post(handlers::create_farm))
            .route("/{id}", delete(handlers::delete_farm))
            .route("/{id}/cells", put(handlers::repaint_farm_cells)),
    )
}
