use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::shared::AppState;

use super::handlers;

/// What the dashboard reads without a login: markets, prices, the board of
/// listings and the deals of a day.
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/alwa/markets", get(handlers::get_markets))
        .route(
            "/alwa/markets/{slug}/prices",
            get(handlers::get_market_prices),
        )
        .route(
            "/alwa/markets/{slug}/prices/{crop}/history",
            get(handlers::get_price_history),
        )
        .route("/alwa/listings", get(handlers::get_listings))
        .route("/alwa/listings/{id}", get(handlers::get_listing))
        .route("/alwa/deals", get(handlers::get_deals))
}

/// What a farmer or a buyer does with their token. `/alwa/listings/mine`
/// lives here and not beside the public `/alwa/listings/{id}`: the fixed
/// path wins over the id, and only this router asks for the token.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/alwa/listings", post(handlers::create_listing))
        .route("/alwa/listings/mine", get(handlers::get_my_listings))
        .route("/alwa/listings/{id}", delete(handlers::cancel_listing))
        .route("/alwa/listings/{id}/offers", post(handlers::create_offer))
        .route(
            "/alwa/listings/{id}/offers/{offer_id}/accept",
            post(handlers::accept_offer),
        )
        .route("/alwa/offers/mine", get(handlers::get_my_offers))
}

/// What the price job writes. Mounted under `/v1/ingest`, behind the
/// service key.
pub fn ingest_routes() -> Router<AppState> {
    Router::new().route(
        "/alwa/markets/{slug}/prices/{crop}/{day}",
        put(handlers::put_price),
    )
}
