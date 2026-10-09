use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;

use super::{
    dashboard_dtos::{
        AlwaModeratedListingResponse, AlwaModeratedListingsResponse, AlwaModerationQuery,
        AlwaOneMarketResponse, AlwaOneModeratedListingResponse, AlwaStoredPricesQuery,
        AlwaStoredPricesResponse, CreateAlwaMarketParams, CreateAlwaPriceParams,
        ModerateAlwaListingParams, UpdateAlwaMarketParams,
    },
    dtos::{
        AlwaMarketResponse, AlwaMarketsResponse, AlwaOnePriceResponse, RecordAlwaPriceParams,
        parse_day,
    },
    errors::WebError,
};

use crate::{
    app::{Pagination, StaffContext},
    features::alwa::domain::{Crop, MarketSlug},
    infra::http::{ApiResponse, ErrorBody, PaginationQueryDto, ValidatedJson},
    shared::AppState,
};

/// Listing ids travel as opaque strings. One that is not a number cannot
/// name anything, so it is not found rather than a bad request.
fn numeric_id(raw: &str) -> Result<i32, WebError> {
    raw.parse().map_err(|_| WebError::not_found())
}

/// List every alwa market
#[utoipa::path(
    get,
    path = "/v1/dashboard/alwa/markets",
    tag = "alwa",
    responses(
        (status = 200, description = "Markets retrieved successfully", body = AlwaMarketsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs alwa:read", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_markets(
    State(state): State<AppState>,
) -> Result<ApiResponse<AlwaMarketsResponse>, WebError> {
    let markets = state.features.alwa.list_markets_use_case.execute().await?;

    Ok(ApiResponse::ok(AlwaMarketsResponse {
        markets: markets.iter().map(AlwaMarketResponse::from).collect(),
    }))
}

/// Add an alwa market
#[utoipa::path(
    post,
    path = "/v1/dashboard/alwa/markets",
    tag = "alwa",
    request_body = CreateAlwaMarketParams,
    responses(
        (status = 201, description = "Market created successfully", body = AlwaOneMarketResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs alwa:create", body = ErrorBody),
        (status = 409, description = "A market has this slug (`already_exists`)", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_market(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    ValidatedJson(params): ValidatedJson<CreateAlwaMarketParams>,
) -> Result<ApiResponse<AlwaOneMarketResponse>, WebError> {
    let input = params.into_input()?;

    let market = state
        .features
        .alwa
        .create_market_use_case
        .execute(*staff_context.staff_id(), input)
        .await?;

    Ok(ApiResponse::created(AlwaOneMarketResponse::from(&market)))
}

/// Rename an alwa market
#[utoipa::path(
    put,
    path = "/v1/dashboard/alwa/markets/{slug}",
    tag = "alwa",
    params(("slug" = String, Path, description = "Market slug")),
    request_body = UpdateAlwaMarketParams,
    responses(
        (status = 200, description = "Market updated successfully", body = AlwaOneMarketResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs alwa:update", body = ErrorBody),
        (status = 404, description = "Market not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_market(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(slug), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<UpdateAlwaMarketParams>,
) -> Result<ApiResponse<AlwaOneMarketResponse>, WebError> {
    let (slug, names) = params.into_input(slug)?;

    let market = state
        .features
        .alwa
        .update_market_use_case
        .execute(*staff_context.staff_id(), slug, names)
        .await?;

    Ok(ApiResponse::ok(AlwaOneMarketResponse::from(&market)))
}

/// Remove an alwa market that has no prices and no listings
#[utoipa::path(
    delete,
    path = "/v1/dashboard/alwa/markets/{slug}",
    tag = "alwa",
    params(("slug" = String, Path, description = "Market slug")),
    responses(
        (status = 204, description = "Market removed, or already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs alwa:delete", body = ErrorBody),
        (status = 409, description = "The market still has prices or listings (`market_in_use`)", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_market(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(slug), _): WithRejection<Path<String>, WebError>,
) -> Result<StatusCode, WebError> {
    let slug = MarketSlug::new(slug).map_err(crate::features::alwa::app::AppError::from)?;

    state
        .features
        .alwa
        .delete_market_use_case
        .execute(*staff_context.staff_id(), slug)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// List the prices stored for one market
#[utoipa::path(
    get,
    path = "/v1/dashboard/alwa/markets/{slug}/prices",
    tag = "alwa",
    params(
        ("slug" = String, Path, description = "Market slug"),
        AlwaStoredPricesQuery,
        PaginationQueryDto
    ),
    responses(
        (status = 200, description = "Prices retrieved successfully", body = AlwaStoredPricesResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs alwa:read", body = ErrorBody),
        (status = 404, description = "Market not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_stored_prices(
    State(state): State<AppState>,
    WithRejection(Path(slug), _): WithRejection<Path<String>, WebError>,
    WithRejection(Query(query), _): WithRejection<Query<AlwaStoredPricesQuery>, WebError>,
    WithRejection(Query(page), _): WithRejection<Query<PaginationQueryDto>, WebError>,
) -> Result<ApiResponse<AlwaStoredPricesResponse>, WebError> {
    let pagination = Pagination::from(&page);
    let input = query.into_input(slug, pagination)?;

    let (market, prices, count) = state
        .features
        .alwa
        .list_stored_prices_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(AlwaStoredPricesResponse::new(
        &market,
        &prices,
        count,
        &pagination,
    )))
}

/// Enter the price of one crop at one market on one day
#[utoipa::path(
    post,
    path = "/v1/dashboard/alwa/markets/{slug}/prices",
    tag = "alwa",
    params(("slug" = String, Path, description = "Market slug")),
    request_body = CreateAlwaPriceParams,
    responses(
        (status = 201, description = "Price created successfully", body = AlwaOnePriceResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs alwa:create", body = ErrorBody),
        (status = 404, description = "Market not found", body = ErrorBody),
        (status = 409, description = "The market has a price for that crop and day (`already_exists`)", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_price(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(slug), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<CreateAlwaPriceParams>,
) -> Result<ApiResponse<AlwaOnePriceResponse>, WebError> {
    let input = params.into_input(slug)?;

    let (market, price) = state
        .features
        .alwa
        .create_price_use_case
        .execute(*staff_context.staff_id(), input)
        .await?;

    Ok(ApiResponse::created(AlwaOnePriceResponse::from((
        &market, &price,
    ))))
}

/// Correct the price of one crop at one market on one day
#[utoipa::path(
    put,
    path = "/v1/dashboard/alwa/markets/{slug}/prices/{crop}/{day}",
    tag = "alwa",
    params(
        ("slug" = String, Path, description = "Market slug"),
        ("crop" = String, Path, description = "Crop code"),
        ("day" = String, Path, description = "Day, YYYY-MM-DD")
    ),
    request_body = RecordAlwaPriceParams,
    responses(
        (status = 200, description = "Price updated successfully", body = AlwaOnePriceResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs alwa:update", body = ErrorBody),
        (status = 404, description = "Market not found, or no price for that crop and day", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_price(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
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
        .update_price_use_case
        .execute(*staff_context.staff_id(), input)
        .await?;

    Ok(ApiResponse::ok(AlwaOnePriceResponse::from((
        &market, &price,
    ))))
}

/// Remove the price of one crop at one market on one day
#[utoipa::path(
    delete,
    path = "/v1/dashboard/alwa/markets/{slug}/prices/{crop}/{day}",
    tag = "alwa",
    params(
        ("slug" = String, Path, description = "Market slug"),
        ("crop" = String, Path, description = "Crop code"),
        ("day" = String, Path, description = "Day, YYYY-MM-DD")
    ),
    responses(
        (status = 204, description = "Price removed, or already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs alwa:delete", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_price(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((slug, crop, day)), _): WithRejection<
        Path<(String, String, String)>,
        WebError,
    >,
) -> Result<StatusCode, WebError> {
    let slug = MarketSlug::new(slug).map_err(crate::features::alwa::app::AppError::from)?;
    let crop = Crop::new(crop.as_str()).map_err(crate::features::alwa::app::AppError::from)?;
    let day = parse_day(&day)?;

    state
        .features
        .alwa
        .delete_price_use_case
        .execute(*staff_context.staff_id(), slug, crop, day)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// List every seller's listings, of every status, for moderation
#[utoipa::path(
    get,
    path = "/v1/dashboard/alwa/listings",
    tag = "alwa",
    params(AlwaModerationQuery, PaginationQueryDto),
    responses(
        (status = 200, description = "Listings retrieved successfully", body = AlwaModeratedListingsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs alwa:read", body = ErrorBody),
        (status = 404, description = "Market not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_listings(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<AlwaModerationQuery>, WebError>,
    WithRejection(Query(page), _): WithRejection<Query<PaginationQueryDto>, WebError>,
) -> Result<ApiResponse<AlwaModeratedListingsResponse>, WebError> {
    let pagination = Pagination::from(&page);
    let input = query.into_input(pagination)?;

    let (cards, count) = state
        .features
        .alwa
        .list_all_listings_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(AlwaModeratedListingsResponse {
        listings: cards
            .iter()
            .map(AlwaModeratedListingResponse::try_from)
            .collect::<Result<Vec<_>, _>>()?,
        count,
        page: *pagination.page(),
        rows_per_page: *pagination.rows_per_page(),
    }))
}

/// Get one listing with all its offers and both sides' phones
#[utoipa::path(
    get,
    path = "/v1/dashboard/alwa/listings/{id}",
    tag = "alwa",
    params(("id" = String, Path, description = "Listing ID")),
    responses(
        (status = 200, description = "Listing retrieved successfully", body = AlwaOneModeratedListingResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs alwa:read", body = ErrorBody),
        (status = 404, description = "Listing not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_listing(
    State(state): State<AppState>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<ApiResponse<AlwaOneModeratedListingResponse>, WebError> {
    let card = state
        .features
        .alwa
        .view_listing_use_case
        .execute(numeric_id(&id)?)
        .await?;

    Ok(ApiResponse::ok(AlwaOneModeratedListingResponse::try_from(
        &card,
    )?))
}

/// Close an open listing and decline its open offers
#[utoipa::path(
    put,
    path = "/v1/dashboard/alwa/listings/{id}",
    tag = "alwa",
    params(("id" = String, Path, description = "Listing ID")),
    request_body = ModerateAlwaListingParams,
    responses(
        (status = 200, description = "Listing closed, or already closed by this same request", body = AlwaOneModeratedListingResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs alwa:update", body = ErrorBody),
        (status = 404, description = "Listing not found", body = ErrorBody),
        (status = 409, description = "The listing is not open (`listing_not_open`)", body = ErrorBody),
        (status = 422, description = "Validation error, or a status other than `closed` (`status_not_allowed`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn moderate_listing(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<ModerateAlwaListingParams>,
) -> Result<ApiResponse<AlwaOneModeratedListingResponse>, WebError> {
    let input = params.into_input(numeric_id(&id)?)?;

    let card = state
        .features
        .alwa
        .moderate_listing_use_case
        .execute(*staff_context.staff_id(), input)
        .await?;

    Ok(ApiResponse::ok(AlwaOneModeratedListingResponse::try_from(
        &card,
    )?))
}

/// Remove a listing and its offers, unless it was sold
#[utoipa::path(
    delete,
    path = "/v1/dashboard/alwa/listings/{id}",
    tag = "alwa",
    params(("id" = String, Path, description = "Listing ID")),
    responses(
        (status = 204, description = "Listing removed, or already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs alwa:delete", body = ErrorBody),
        (status = 409, description = "An offer on the listing was accepted (`listing_has_deal`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_listing(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<StatusCode, WebError> {
    // An id that is not a number names nothing, and nothing is already gone.
    let Ok(id) = id.parse::<i32>() else {
        return Ok(StatusCode::NO_CONTENT);
    };

    state
        .features
        .alwa
        .delete_listing_use_case
        .execute(*staff_context.staff_id(), id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
