use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::{
    app::{Action, Resource},
    require,
    shared::AppState,
};

use super::{dashboard_handlers, handlers};

/// What anyone reads without a login: markets, prices, the board of
/// listings and the deals of a day. The two listing routes look at a
/// farmer's token when there is one, to show the seller's phone.
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
        .route(
            "/alwa/listings/{id}/sold",
            post(handlers::mark_listing_sold),
        )
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

/// What Ministry staff do on the dashboard. Mounted under `/v1/dashboard`,
/// behind the `staff_auth` layer; each method carries the one permission it
/// needs. There is no create for listings or offers: only farmers and
/// buyers make those.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/alwa/markets",
            get(dashboard_handlers::get_markets)
                .route_layer(require!(Resource::Alwa, Action::Read))
                .merge(
                    post(dashboard_handlers::create_market)
                        .route_layer(require!(Resource::Alwa, Action::Create)),
                ),
        )
        .route(
            "/alwa/markets/{slug}",
            put(dashboard_handlers::update_market)
                .route_layer(require!(Resource::Alwa, Action::Update))
                .merge(
                    delete(dashboard_handlers::delete_market)
                        .route_layer(require!(Resource::Alwa, Action::Delete)),
                ),
        )
        .route(
            "/alwa/markets/{slug}/prices",
            get(dashboard_handlers::get_stored_prices)
                .route_layer(require!(Resource::Alwa, Action::Read))
                .merge(
                    post(dashboard_handlers::create_price)
                        .route_layer(require!(Resource::Alwa, Action::Create)),
                ),
        )
        .route(
            "/alwa/markets/{slug}/prices/{crop}/{day}",
            put(dashboard_handlers::update_price)
                .route_layer(require!(Resource::Alwa, Action::Update))
                .merge(
                    delete(dashboard_handlers::delete_price)
                        .route_layer(require!(Resource::Alwa, Action::Delete)),
                ),
        )
        .route(
            "/alwa/listings",
            get(dashboard_handlers::get_listings)
                .route_layer(require!(Resource::Alwa, Action::Read)),
        )
        .route(
            "/alwa/listings/{id}",
            get(dashboard_handlers::get_listing)
                .route_layer(require!(Resource::Alwa, Action::Read))
                .merge(
                    put(dashboard_handlers::moderate_listing)
                        .route_layer(require!(Resource::Alwa, Action::Update)),
                )
                .merge(
                    delete(dashboard_handlers::delete_listing)
                        .route_layer(require!(Resource::Alwa, Action::Delete)),
                ),
        )
}
