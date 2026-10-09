use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        CompareQueryDto, CreateZoneDashboardReadingParams, CreateZoneDashboardSubZoneReadingParams,
        MonthQueryDto, RegionComparisonResponse, RegionOverviewResponse, SubZoneReadingParams,
        SubZoneReadingResponse, ZoneDashboardReadingsQuery, ZoneDashboardReadingsResponse,
        ZoneDashboardSubZoneReadingsResponse, ZoneDashboardZoneResponse,
        ZoneDashboardZonesResponse, ZoneDetailResponse, ZoneReadingParams, ZoneReadingResponse,
    },
    errors::WebError,
};

use crate::{
    app::{Pagination, StaffContext},
    features::zones::{
        app::{
            AppError,
            use_cases::{ListSubZoneReadingsInput, ListZoneReadingsInput},
        },
        domain::{Month, ZoneSlug},
    },
    infra::http::{ApiResponse, ErrorBody, PaginationQueryDto, ValidatedJson},
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

/// List every zone with its sub-zones, for the dashboard's pickers
#[utoipa::path(
    get,
    path = "/v1/dashboard/zones",
    tag = "zones",
    responses(
        (status = 200, description = "Zones retrieved successfully", body = ZoneDashboardZonesResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs zones:read", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard_zones(
    State(state): State<AppState>,
) -> Result<ApiResponse<ZoneDashboardZonesResponse>, WebError> {
    let zones = state.features.zone.list_zones_use_case.execute().await?;

    Ok(ApiResponse::ok(ZoneDashboardZonesResponse {
        zones: zones.iter().map(ZoneDashboardZoneResponse::from).collect(),
    }))
}

/// List a zone's stored readings, newest month first
#[utoipa::path(
    get,
    path = "/v1/dashboard/zones/{slug}/readings",
    tag = "zones",
    params(
        ("slug" = String, Path, description = "Zone slug"),
        ("from" = Option<String>, Query, description = "YYYY-MM, included. Default: 23 months before `to`"),
        ("to" = Option<String>, Query, description = "YYYY-MM, included. Default: the current month"),
        PaginationQueryDto
    ),
    responses(
        (status = 200, description = "Readings retrieved successfully", body = ZoneDashboardReadingsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs zones:read", body = ErrorBody),
        (status = 404, description = "Zone not found", body = ErrorBody),
        (status = 422, description = "Validation error: `bad_month`, or `bad_range` when `from` is after `to`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard_zone_readings(
    State(state): State<AppState>,
    WithRejection(Path(slug), _): WithRejection<Path<String>, WebError>,
    WithRejection(Query(query), _): WithRejection<Query<ZoneDashboardReadingsQuery>, WebError>,
    WithRejection(Query(page), _): WithRejection<Query<PaginationQueryDto>, WebError>,
) -> Result<ApiResponse<ZoneDashboardReadingsResponse>, WebError> {
    let slug = zone_slug(slug)?;
    let (from, to) = query.into_bounds()?;
    let pagination = Pagination::from(&page);

    let (readings, count) = state
        .features
        .zone
        .list_zone_readings_use_case
        .execute(ListZoneReadingsInput {
            zone_slug: slug.clone(),
            from,
            to,
            pagination,
        })
        .await?;

    Ok(ApiResponse::ok(ZoneDashboardReadingsResponse {
        readings: readings
            .iter()
            .map(|reading| ZoneReadingResponse::from((&slug, reading)))
            .collect(),
        count,
        page: *pagination.page(),
        rows_per_page: *pagination.rows_per_page(),
    }))
}

/// Create a zone's reading for a month that has none
#[utoipa::path(
    post,
    path = "/v1/dashboard/zones/{slug}/readings",
    tag = "zones",
    params(("slug" = String, Path, description = "Zone slug")),
    request_body = CreateZoneDashboardReadingParams,
    responses(
        (status = 201, description = "Reading created", body = ZoneReadingResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs zones:create", body = ErrorBody),
        (status = 404, description = "Zone not found", body = ErrorBody),
        (status = 409, description = "The zone already has a reading for that month (`already_exists`)", body = ErrorBody),
        (status = 422, description = "Validation error, for example `bad_month` or `bad_dryness`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_dashboard_zone_reading(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(slug), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<CreateZoneDashboardReadingParams>,
) -> Result<ApiResponse<ZoneReadingResponse>, WebError> {
    let slug = zone_slug(slug)?;
    let input = params.into_input(slug.clone())?;

    let stored = state
        .features
        .zone
        .create_zone_reading_use_case
        .execute(&staff_context, input)
        .await?;

    Ok(ApiResponse::created(ZoneReadingResponse::from((
        &slug, &stored,
    ))))
}

/// Replace a zone's stored reading for a month
#[utoipa::path(
    put,
    path = "/v1/dashboard/zones/{slug}/readings/{month}",
    tag = "zones",
    params(
        ("slug" = String, Path, description = "Zone slug"),
        ("month" = String, Path, description = "YYYY-MM")
    ),
    request_body = ZoneReadingParams,
    responses(
        (status = 200, description = "Reading updated", body = ZoneReadingResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs zones:update", body = ErrorBody),
        (status = 404, description = "Zone not found, or it has no reading for that month", body = ErrorBody),
        (status = 422, description = "Validation error, for example `bad_month` or `bad_dryness`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_dashboard_zone_reading(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((slug, month)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<ZoneReadingParams>,
) -> Result<ApiResponse<ZoneReadingResponse>, WebError> {
    let slug = zone_slug(slug)?;
    let input = params.into_input(slug.clone(), month_in_path(&month)?)?;

    let stored = state
        .features
        .zone
        .update_zone_reading_use_case
        .execute(&staff_context, input)
        .await?;

    Ok(ApiResponse::ok(ZoneReadingResponse::from((&slug, &stored))))
}

/// Delete a zone's reading for a month
#[utoipa::path(
    delete,
    path = "/v1/dashboard/zones/{slug}/readings/{month}",
    tag = "zones",
    params(
        ("slug" = String, Path, description = "Zone slug"),
        ("month" = String, Path, description = "YYYY-MM")
    ),
    responses(
        (status = 204, description = "Deleted, or already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs zones:delete", body = ErrorBody),
        (status = 404, description = "Zone not found", body = ErrorBody),
        (status = 422, description = "Validation error: `bad_month`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_dashboard_zone_reading(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((slug, month)), _): WithRejection<Path<(String, String)>, WebError>,
) -> Result<StatusCode, WebError> {
    let slug = zone_slug(slug)?;

    state
        .features
        .zone
        .delete_zone_reading_use_case
        .execute(&staff_context, &slug, month_in_path(&month)?)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// List a sub-zone's stored readings, newest month first
#[utoipa::path(
    get,
    path = "/v1/dashboard/zones/{slug}/sub-zones/{sub_slug}/readings",
    tag = "zones",
    params(
        ("slug" = String, Path, description = "Zone slug"),
        ("sub_slug" = String, Path, description = "Sub-zone slug, unique inside its zone"),
        ("from" = Option<String>, Query, description = "YYYY-MM, included. Default: 23 months before `to`"),
        ("to" = Option<String>, Query, description = "YYYY-MM, included. Default: the current month"),
        PaginationQueryDto
    ),
    responses(
        (status = 200, description = "Readings retrieved successfully", body = ZoneDashboardSubZoneReadingsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs zones:read", body = ErrorBody),
        (status = 404, description = "Zone or sub-zone not found", body = ErrorBody),
        (status = 422, description = "Validation error: `bad_month`, or `bad_range` when `from` is after `to`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard_sub_zone_readings(
    State(state): State<AppState>,
    WithRejection(Path((slug, sub_slug)), _): WithRejection<Path<(String, String)>, WebError>,
    WithRejection(Query(query), _): WithRejection<Query<ZoneDashboardReadingsQuery>, WebError>,
    WithRejection(Query(page), _): WithRejection<Query<PaginationQueryDto>, WebError>,
) -> Result<ApiResponse<ZoneDashboardSubZoneReadingsResponse>, WebError> {
    let slug = zone_slug(slug)?;
    let sub_slug = zone_slug(sub_slug)?;
    let (from, to) = query.into_bounds()?;
    let pagination = Pagination::from(&page);

    let (readings, count) = state
        .features
        .zone
        .list_sub_zone_readings_use_case
        .execute(ListSubZoneReadingsInput {
            zone_slug: slug.clone(),
            sub_zone_slug: sub_slug.clone(),
            from,
            to,
            pagination,
        })
        .await?;

    Ok(ApiResponse::ok(ZoneDashboardSubZoneReadingsResponse {
        readings: readings
            .iter()
            .map(|reading| SubZoneReadingResponse::from((&slug, &sub_slug, reading)))
            .collect(),
        count,
        page: *pagination.page(),
        rows_per_page: *pagination.rows_per_page(),
    }))
}

/// Create a sub-zone's reading for a month that has none
#[utoipa::path(
    post,
    path = "/v1/dashboard/zones/{slug}/sub-zones/{sub_slug}/readings",
    tag = "zones",
    params(
        ("slug" = String, Path, description = "Zone slug"),
        ("sub_slug" = String, Path, description = "Sub-zone slug, unique inside its zone")
    ),
    request_body = CreateZoneDashboardSubZoneReadingParams,
    responses(
        (status = 201, description = "Reading created", body = SubZoneReadingResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs zones:create", body = ErrorBody),
        (status = 404, description = "Zone or sub-zone not found", body = ErrorBody),
        (status = 409, description = "The sub-zone already has a reading for that month (`already_exists`)", body = ErrorBody),
        (status = 422, description = "Validation error: `bad_month` or `bad_dryness`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_dashboard_sub_zone_reading(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((slug, sub_slug)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<CreateZoneDashboardSubZoneReadingParams>,
) -> Result<ApiResponse<SubZoneReadingResponse>, WebError> {
    let slug = zone_slug(slug)?;
    let sub_slug = zone_slug(sub_slug)?;
    let input = params.into_input(slug.clone(), sub_slug.clone())?;

    let stored = state
        .features
        .zone
        .create_sub_zone_reading_use_case
        .execute(&staff_context, input)
        .await?;

    Ok(ApiResponse::created(SubZoneReadingResponse::from((
        &slug, &sub_slug, &stored,
    ))))
}

/// Replace a sub-zone's stored dryness for a month
#[utoipa::path(
    put,
    path = "/v1/dashboard/zones/{slug}/sub-zones/{sub_slug}/readings/{month}",
    tag = "zones",
    params(
        ("slug" = String, Path, description = "Zone slug"),
        ("sub_slug" = String, Path, description = "Sub-zone slug, unique inside its zone"),
        ("month" = String, Path, description = "YYYY-MM")
    ),
    request_body = SubZoneReadingParams,
    responses(
        (status = 200, description = "Reading updated", body = SubZoneReadingResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs zones:update", body = ErrorBody),
        (status = 404, description = "Zone or sub-zone not found, or it has no reading for that month", body = ErrorBody),
        (status = 422, description = "Validation error: `bad_month` or `bad_dryness`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_dashboard_sub_zone_reading(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
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
        .update_sub_zone_reading_use_case
        .execute(&staff_context, input)
        .await?;

    Ok(ApiResponse::ok(SubZoneReadingResponse::from((
        &slug, &sub_slug, &stored,
    ))))
}

/// Delete a sub-zone's reading for a month
#[utoipa::path(
    delete,
    path = "/v1/dashboard/zones/{slug}/sub-zones/{sub_slug}/readings/{month}",
    tag = "zones",
    params(
        ("slug" = String, Path, description = "Zone slug"),
        ("sub_slug" = String, Path, description = "Sub-zone slug, unique inside its zone"),
        ("month" = String, Path, description = "YYYY-MM")
    ),
    responses(
        (status = 204, description = "Deleted, or already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs zones:delete", body = ErrorBody),
        (status = 404, description = "Zone or sub-zone not found", body = ErrorBody),
        (status = 422, description = "Validation error: `bad_month`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_dashboard_sub_zone_reading(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((slug, sub_slug, month)), _): WithRejection<
        Path<(String, String, String)>,
        WebError,
    >,
) -> Result<StatusCode, WebError> {
    let slug = zone_slug(slug)?;
    let sub_slug = zone_slug(sub_slug)?;

    state
        .features
        .zone
        .delete_sub_zone_reading_use_case
        .execute(&staff_context, &slug, &sub_slug, month_in_path(&month)?)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
