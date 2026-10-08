use axum::extract::{Path, Query, State};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        CompareQueryDto, MonthQueryDto, RegionComparisonResponse, RegionOverviewResponse,
        SubZoneReadingParams, SubZoneReadingResponse, ZoneDetailResponse, ZoneReadingParams,
        ZoneReadingResponse,
    },
    errors::WebError,
};

use crate::{
    features::zones::{
        app::AppError,
        domain::{Month, ZoneSlug},
    },
    infra::http::{ApiResponse, ErrorBody, ValidatedJson},
    shared::AppState,
};

/// A slug that is not written like one cannot name a zone, so it is not
/// found rather than a bad request.
fn zone_slug(raw: String) -> Result<ZoneSlug, WebError> {
    ZoneSlug::new(raw).map_err(|_| WebError::not_found())
}

fn month_in_path(raw: &str) -> Result<Month, WebError> {
    Ok(Month::parse(raw).map_err(AppError::from)?)
}

/// The whole region for one month: every zone and a summary
#[utoipa::path(
    get,
    path = "/v1/region/overview",
    tag = "zones",
    params(("month" = Option<String>, Query, description = "YYYY-MM. Default: the latest month that has any zone reading")),
    responses(
        (status = 200, description = "Region overview retrieved successfully", body = RegionOverviewResponse),
        (status = 422, description = "Validation error, for example `bad_month`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_region_overview(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<MonthQueryDto>, WebError>,
) -> Result<ApiResponse<RegionOverviewResponse>, WebError> {
    let month = query.into_month()?;

    let overview = state
        .features
        .zone
        .view_region_overview_use_case
        .execute(month)
        .await?;

    Ok(ApiResponse::ok(RegionOverviewResponse::from(&overview)))
}

/// One zone for one month, with its sub-zones and its history
#[utoipa::path(
    get,
    path = "/v1/zones/{slug}",
    tag = "zones",
    params(
        ("slug" = String, Path, description = "Zone slug, for example `chamchamal`"),
        ("month" = Option<String>, Query, description = "YYYY-MM. Default: the latest month that has any zone reading")
    ),
    responses(
        (status = 200, description = "Zone retrieved successfully", body = ZoneDetailResponse),
        (status = 404, description = "Zone not found", body = ErrorBody),
        (status = 422, description = "Validation error, for example `bad_month`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_zone(
    State(state): State<AppState>,
    WithRejection(Path(slug), _): WithRejection<Path<String>, WebError>,
    WithRejection(Query(query), _): WithRejection<Query<MonthQueryDto>, WebError>,
) -> Result<ApiResponse<ZoneDetailResponse>, WebError> {
    let slug = zone_slug(slug)?;
    let month = query.into_month()?;

    let detail = state
        .features
        .zone
        .view_zone_use_case
        .execute(&slug, month)
        .await?;

    Ok(ApiResponse::ok(ZoneDetailResponse::from(&detail)))
}

/// One calendar month compared between two years
#[utoipa::path(
    get,
    path = "/v1/region/compare",
    tag = "zones",
    params(
        ("year" = i32, Query, description = "The year to show"),
        ("with" = i32, Query, description = "The year to compare it with. Must differ from `year`"),
        ("month" = u32, Query, description = "Calendar month, 1 to 12")
    ),
    responses(
        (status = 200, description = "Comparison retrieved successfully", body = RegionComparisonResponse),
        (status = 422, description = "Validation error: `bad_year`, `bad_month` or `same_year`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_region_comparison(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<CompareQueryDto>, WebError>,
) -> Result<ApiResponse<RegionComparisonResponse>, WebError> {
    let comparison = query.into_input()?;

    let compared = state
        .features
        .zone
        .compare_years_use_case
        .execute(comparison)
        .await?;

    Ok(ApiResponse::ok(RegionComparisonResponse::from(&compared)))
}

/// Store a zone's reading for a month, replacing the one already there
#[utoipa::path(
    put,
    path = "/v1/ingest/zones/{slug}/readings/{month}",
    tag = "zones",
    params(
        ("slug" = String, Path, description = "Zone slug"),
        ("month" = String, Path, description = "YYYY-MM"),
        ("X-Service-Key" = String, Header, description = "The data jobs' shared key")
    ),
    request_body = ZoneReadingParams,
    responses(
        (status = 200, description = "Reading stored", body = ZoneReadingResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 404, description = "Zone not found", body = ErrorBody),
        (status = 422, description = "Validation error, for example `bad_month` or `bad_dryness`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_zone_reading(
    State(state): State<AppState>,
    WithRejection(Path((slug, month)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<ZoneReadingParams>,
) -> Result<ApiResponse<ZoneReadingResponse>, WebError> {
    let slug = zone_slug(slug)?;
    let input = params.into_input(slug.clone(), month_in_path(&month)?)?;

    let stored = state
        .features
        .zone
        .record_zone_reading_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(ZoneReadingResponse::from((&slug, &stored))))
}

/// Store a sub-zone's dryness for a month, replacing the one already there
#[utoipa::path(
    put,
    path = "/v1/ingest/zones/{slug}/sub-zones/{sub_slug}/readings/{month}",
    tag = "zones",
    params(
        ("slug" = String, Path, description = "Zone slug"),
        ("sub_slug" = String, Path, description = "Sub-zone slug, unique inside its zone"),
        ("month" = String, Path, description = "YYYY-MM"),
        ("X-Service-Key" = String, Header, description = "The data jobs' shared key")
    ),
    request_body = SubZoneReadingParams,
    responses(
        (status = 200, description = "Reading stored", body = SubZoneReadingResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 404, description = "Zone or sub-zone not found", body = ErrorBody),
        (status = 422, description = "Validation error, for example `bad_month` or `bad_dryness`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_sub_zone_reading(
    State(state): State<AppState>,
    WithRejection(Path((slug, sub_slug, month)), _): WithRejection<
        Path<(String, String, String)>,
        WebError,
    >,
    ValidatedJson(params): ValidatedJson<SubZoneReadingParams>,
) -> Result<ApiResponse<SubZoneReadingResponse>, WebError> {
    let slug = zone_slug(slug)?;
    let sub_slug = zone_slug(sub_slug)?;
    let input = params.into_input(slug.clone(), sub_slug.clone(), month_in_path(&month)?)?;

    let stored = state
        .features
        .zone
        .record_sub_zone_reading_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(SubZoneReadingResponse::from((
        &slug, &sub_slug, &stored,
    ))))
}
