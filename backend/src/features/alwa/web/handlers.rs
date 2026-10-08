use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        AlwaDealsQuery, AlwaDealsResponse, AlwaHistoryQuery, AlwaListingSummaryResponse,
        AlwaListingsQuery, AlwaListingsResponse, AlwaMarketPricesResponse, AlwaMarketResponse,
        AlwaMarketsResponse, AlwaMyListingsResponse, AlwaMyOffersResponse, AlwaOneListingResponse,
        AlwaOneOfferResponse, AlwaOnePriceResponse, AlwaPriceHistoryResponse, AlwaPricesQuery,
        MakeAlwaOfferParams, PostAlwaListingParams, RecordAlwaPriceParams,
    },
    errors::WebError,
};

use crate::{
    app::{AuthContext, Pagination},
    infra::http::{ApiResponse, ErrorBody, PaginationQueryDto, ValidatedJson},
    shared::AppState,
};

/// Listing and offer ids travel as opaque strings. One that is not a number
/// cannot name anything, so it is not found rather than a bad request.
fn numeric_id(raw: &str) -> Result<i32, WebError> {
    raw.parse().map_err(|_| WebError::not_found())
}

/// List the alwa markets
#[utoipa::path(
    get,
    path = "/v1/alwa/markets",
    tag = "alwa",
    responses(
        (status = 200, description = "Markets retrieved successfully", body = AlwaMarketsResponse),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_markets(
    State(state): State<AppState>,
) -> Result<ApiResponse<AlwaMarketsResponse>, WebError> {
    let markets = state.features.alwa.list_markets_use_case.execute().await?;

    Ok(ApiResponse::ok(AlwaMarketsResponse {
        markets: markets.iter().map(AlwaMarketResponse::from).collect(),
    }))
}

/// Get the prices of every crop at one market on one day
#[utoipa::path(
    get,
    path = "/v1/alwa/markets/{slug}/prices",
    tag = "alwa",
    params(("slug" = String, Path, description = "Market slug"), AlwaPricesQuery),
    responses(
        (status = 200, description = "Prices retrieved successfully", body = AlwaMarketPricesResponse),
        (status = 404, description = "Market not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_market_prices(
    State(state): State<AppState>,
    WithRejection(Path(slug), _): WithRejection<Path<String>, WebError>,
    WithRejection(Query(query), _): WithRejection<Query<AlwaPricesQuery>, WebError>,
) -> Result<ApiResponse<AlwaMarketPricesResponse>, WebError> {
    let input = query.into_input(slug)?;

    let board = state
        .features
        .alwa
        .view_market_prices_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(AlwaMarketPricesResponse::from(&board)))
}

/// Get the price history of one crop at one market
#[utoipa::path(
    get,
    path = "/v1/alwa/markets/{slug}/prices/{crop}/history",
    tag = "alwa",
    params(
        ("slug" = String, Path, description = "Market slug"),
        ("crop" = String, Path, description = "Crop code"),
        AlwaHistoryQuery
    ),
    responses(
        (status = 200, description = "History retrieved successfully", body = AlwaPriceHistoryResponse),
        (status = 404, description = "Market not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_price_history(
    State(state): State<AppState>,
    WithRejection(Path((slug, crop)), _): WithRejection<Path<(String, String)>, WebError>,
    WithRejection(Query(query), _): WithRejection<Query<AlwaHistoryQuery>, WebError>,
) -> Result<ApiResponse<AlwaPriceHistoryResponse>, WebError> {
    let input = query.into_input(slug, &crop)?;
    let crop = input.crop;

    let (market, history) = state
        .features
        .alwa
        .view_price_history_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(AlwaPriceHistoryResponse {
        market: market.slug().into(),
        crop: crop.into(),
        history: history.iter().map(Into::into).collect(),
    }))
}

/// Browse the listings on sale
#[utoipa::path(
    get,
    path = "/v1/alwa/listings",
    tag = "alwa",
    params(AlwaListingsQuery, PaginationQueryDto),
    responses(
        (status = 200, description = "Listings retrieved successfully", body = AlwaListingsResponse),
        (status = 404, description = "Market not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_listings(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<AlwaListingsQuery>, WebError>,
    WithRejection(Query(page), _): WithRejection<Query<PaginationQueryDto>, WebError>,
) -> Result<ApiResponse<AlwaListingsResponse>, WebError> {
    let pagination = Pagination::from(&page);
    let input = query.into_input(pagination)?;

    let (cards, count) = state
        .features
        .alwa
        .browse_listings_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(AlwaListingsResponse {
        listings: cards
            .iter()
            .map(AlwaListingSummaryResponse::try_from)
            .collect::<Result<Vec<_>, _>>()?,
        count,
        page: *pagination.page(),
        rows_per_page: *pagination.rows_per_page(),
    }))
}

/// Get one listing with its offers
#[utoipa::path(
    get,
    path = "/v1/alwa/listings/{id}",
    tag = "alwa",
    params(("id" = String, Path, description = "Listing ID")),
    responses(
        (status = 200, description = "Listing retrieved successfully", body = AlwaOneListingResponse),
        (status = 404, description = "Listing not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_listing(
    State(state): State<AppState>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<ApiResponse<AlwaOneListingResponse>, WebError> {
    let card = state
        .features
        .alwa
        .view_listing_use_case
        .execute(numeric_id(&id)?)
        .await?;

    // No viewer: this route is public, so it never shows a phone.
    Ok(ApiResponse::ok(AlwaOneListingResponse::new(&card, None)?))
}

/// List the deals made on one day
#[utoipa::path(
    get,
    path = "/v1/alwa/deals",
    tag = "alwa",
    params(AlwaDealsQuery),
    responses(
        (status = 200, description = "Deals retrieved successfully", body = AlwaDealsResponse),
        (status = 404, description = "Market not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_deals(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<AlwaDealsQuery>, WebError>,
) -> Result<ApiResponse<AlwaDealsResponse>, WebError> {
    let input = query.into_input()?;

    let (day, deals) = state
        .features
        .alwa
        .list_deals_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(AlwaDealsResponse::try_from((
        day,
        deals.as_slice(),
    ))?))
}

/// Put a crop on sale
#[utoipa::path(
    post,
    path = "/v1/alwa/listings",
    tag = "alwa",
    request_body = PostAlwaListingParams,
    responses(
        (status = 201, description = "Listing created successfully", body = AlwaOneListingResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Market not found", body = ErrorBody),
        (status = 422, description = "Validation error, or too many open listings", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_listing(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    ValidatedJson(params): ValidatedJson<PostAlwaListingParams>,
) -> Result<ApiResponse<AlwaOneListingResponse>, WebError> {
    let input = params.into_input()?;

    let card = state
        .features
        .alwa
        .post_listing_use_case
        .execute(&auth_context, input)
        .await?;

    Ok(ApiResponse::created(AlwaOneListingResponse::new(
        &card,
        Some(auth_context.user().phone()),
    )?))
}

/// List the authenticated user's own listings with their offers
#[utoipa::path(
    get,
    path = "/v1/alwa/listings/mine",
    tag = "alwa",
    responses(
        (status = 200, description = "Listings retrieved successfully", body = AlwaMyListingsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_my_listings(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
) -> Result<ApiResponse<AlwaMyListingsResponse>, WebError> {
    let cards = state
        .features
        .alwa
        .list_my_listings_use_case
        .execute(&auth_context)
        .await?;

    Ok(ApiResponse::ok(AlwaMyListingsResponse::new(
        &cards,
        auth_context.user().phone(),
    )?))
}

/// Cancel an open listing of the authenticated user
#[utoipa::path(
    delete,
    path = "/v1/alwa/listings/{id}",
    tag = "alwa",
    params(("id" = String, Path, description = "Listing ID")),
    responses(
        (status = 204, description = "Listing cancelled"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Listing not found, or not the caller's", body = ErrorBody),
        (status = 409, description = "The listing is not open", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn cancel_listing(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<StatusCode, WebError> {
    state
        .features
        .alwa
        .cancel_listing_use_case
        .execute(&auth_context, numeric_id(&id)?)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Make an offer on a listing
#[utoipa::path(
    post,
    path = "/v1/alwa/listings/{id}/offers",
    tag = "alwa",
    params(("id" = String, Path, description = "Listing ID")),
    request_body = MakeAlwaOfferParams,
    responses(
        (status = 201, description = "Offer made successfully", body = AlwaOneOfferResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Listing not found", body = ErrorBody),
        (status = 409, description = "The listing is not open", body = ErrorBody),
        (status = 422, description = "Validation error, or the caller's own listing", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_offer(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<MakeAlwaOfferParams>,
) -> Result<ApiResponse<AlwaOneOfferResponse>, WebError> {
    let draft = params.into_input()?;

    let offer = state
        .features
        .alwa
        .make_offer_use_case
        .execute(&auth_context, numeric_id(&id)?, draft)
        .await?;

    Ok(ApiResponse::created(AlwaOneOfferResponse::try_from(
        &offer,
    )?))
}

/// Accept an offer on a listing of the authenticated user
#[utoipa::path(
    post,
    path = "/v1/alwa/listings/{id}/offers/{offer_id}/accept",
    tag = "alwa",
    params(
        ("id" = String, Path, description = "Listing ID"),
        ("offer_id" = String, Path, description = "Offer ID")
    ),
    responses(
        (status = 200, description = "Offer accepted, the listing is sold", body = AlwaOneListingResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Listing not the caller's, or offer not on it", body = ErrorBody),
        (status = 409, description = "The listing or the offer is not open", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn accept_offer(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path((id, offer_id)), _): WithRejection<Path<(String, String)>, WebError>,
) -> Result<ApiResponse<AlwaOneListingResponse>, WebError> {
    let card = state
        .features
        .alwa
        .accept_offer_use_case
        .execute(&auth_context, numeric_id(&id)?, numeric_id(&offer_id)?)
        .await?;

    Ok(ApiResponse::ok(AlwaOneListingResponse::new(
        &card,
        Some(auth_context.user().phone()),
    )?))
}

/// List the authenticated user's own offers
#[utoipa::path(
    get,
    path = "/v1/alwa/offers/mine",
    tag = "alwa",
    responses(
        (status = 200, description = "Offers retrieved successfully", body = AlwaMyOffersResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_my_offers(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
) -> Result<ApiResponse<AlwaMyOffersResponse>, WebError> {
    let placed = state
        .features
        .alwa
        .list_my_offers_use_case
        .execute(&auth_context)
        .await?;

    Ok(ApiResponse::ok(AlwaMyOffersResponse::new(
        &placed,
        auth_context.user().phone(),
    )?))
}

/// Store the price of one crop at one market on one day
#[utoipa::path(
    put,
    path = "/v1/ingest/alwa/markets/{slug}/prices/{crop}/{day}",
    tag = "alwa",
    params(
        ("slug" = String, Path, description = "Market slug"),
        ("crop" = String, Path, description = "Crop code"),
        ("day" = String, Path, description = "Day, YYYY-MM-DD"),
        ("X-Service-Key" = String, Header, description = "The data jobs' shared key")
    ),
    request_body = RecordAlwaPriceParams,
    responses(
        (status = 200, description = "Price stored", body = AlwaOnePriceResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 404, description = "Market not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_price(
    State(state): State<AppState>,
    WithRejection(Path((slug, crop, day)), _): WithRejection<
        Path<(String, String, String)>,
        WebError,
    >,
    ValidatedJson(params): ValidatedJson<RecordAlwaPriceParams>,
) -> Result<ApiResponse<AlwaOnePriceResponse>, WebError> {
    let input = params.into_input(slug, &crop, &day)?;

    let (market, price) = state
        .features
        .alwa
        .record_price_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(AlwaOnePriceResponse::from((
        &market, &price,
    ))))
}
