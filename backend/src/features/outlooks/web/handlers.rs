use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        CreateOutlookDashboardParams, CreateOutlookRunDashboardParams, OutlookDashboardQuery,
        OutlookRunsResponse, OutlooksPageResponse, RecordOutlookRunParams, RecordZoneOutlookParams,
        SavedOutlookRunResponse, SavedZoneOutlookResponse, SeasonOutlookQuery,
        SeasonOutlookResponse, ZoneOutlookHistoryResponse, ZoneOutlookQuery,
    },
    errors::WebError,
};

use crate::{
    app::{Pagination, StaffContext},
    features::outlooks::{
        app::AppError,
        domain::{IssueMonth, Season, ZoneSlug},
    },
    infra::http::{ApiResponse, ErrorBody, PaginationQueryDto, ValidatedJson},
    shared::AppState,
};

/// Get the outlook for the next growing season, zone by zone
#[utoipa::path(
    get,
    path = "/v1/outlooks",
    tag = "outlooks",
    params(SeasonOutlookQuery),
    responses(
        (status = 200, description = "Outlook retrieved successfully", body = SeasonOutlookResponse),
        (status = 404, description = "No outlook issued for that season or month", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_season_outlook(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<SeasonOutlookQuery>, WebError>,
) -> Result<ApiResponse<SeasonOutlookResponse>, WebError> {
    let board = state
        .features
        .outlook
        .view_season_outlook_use_case
        .execute(query.into_input()?)
        .await?;

    Ok(ApiResponse::ok(SeasonOutlookResponse::from(&board)))
}

/// Get one zone's outlook at every issue of a season, oldest first
#[utoipa::path(
    get,
    path = "/v1/outlooks/zones/{zone_slug}",
    tag = "outlooks",
    params(
        ("zone_slug" = String, Path, description = "Zone slug, for example chamchamal"),
        ZoneOutlookQuery
    ),
    responses(
        (status = 200, description = "Zone outlook retrieved successfully", body = ZoneOutlookHistoryResponse),
        (status = 404, description = "No outlook issued yet", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_zone_outlook(
    State(state): State<AppState>,
    WithRejection(Path(zone_slug), _): WithRejection<Path<String>, WebError>,
    WithRejection(Query(query), _): WithRejection<Query<ZoneOutlookQuery>, WebError>,
) -> Result<ApiResponse<ZoneOutlookHistoryResponse>, WebError> {
    let (season, history) = state
        .features
        .outlook
        .view_zone_outlook_use_case
        .execute(query.into_input(zone_slug.clone())?)
        .await?;

    Ok(ApiResponse::ok(ZoneOutlookHistoryResponse {
        zone_slug,
        season: (&season).into(),
        issues: history.iter().map(Into::into).collect(),
    }))
}

/// Store one zone's outlook for one issue, replacing an earlier one
#[utoipa::path(
    put,
    path = "/v1/ingest/outlooks/{season}/{issued}/zones/{zone_slug}",
    tag = "outlooks",
    params(
        ("season" = String, Path, description = "Season, for example 2026-27"),
        ("issued" = String, Path, description = "Issue month, YYYY-MM"),
        ("zone_slug" = String, Path, description = "Zone slug, for example chamchamal"),
        ("X-Service-Key" = String, Header, description = "The data jobs' shared key")
    ),
    request_body = RecordZoneOutlookParams,
    responses(
        (status = 200, description = "Outlook stored", body = SavedZoneOutlookResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_zone_outlook(
    State(state): State<AppState>,
    WithRejection(Path((season, issued, zone_slug)), _): WithRejection<
        Path<(String, String, String)>,
        WebError,
    >,
    ValidatedJson(params): ValidatedJson<RecordZoneOutlookParams>,
) -> Result<ApiResponse<SavedZoneOutlookResponse>, WebError> {
    let input = params.into_input(season, issued, zone_slug)?;

    let outlook = state
        .features
        .outlook
        .record_zone_outlook_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(SavedZoneOutlookResponse::from(&outlook)))
}

/// Store the track record of the method behind one issue
#[utoipa::path(
    put,
    path = "/v1/ingest/outlooks/{season}/{issued}/run",
    tag = "outlooks",
    params(
        ("season" = String, Path, description = "Season, for example 2026-27"),
        ("issued" = String, Path, description = "Issue month, YYYY-MM"),
        ("X-Service-Key" = String, Header, description = "The data jobs' shared key")
    ),
    request_body = RecordOutlookRunParams,
    responses(
        (status = 200, description = "Track record stored", body = SavedOutlookRunResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_outlook_run(
    State(state): State<AppState>,
    WithRejection(Path((season, issued)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<RecordOutlookRunParams>,
) -> Result<ApiResponse<SavedOutlookRunResponse>, WebError> {
    let input = params.into_input(season, issued)?;

    let run = state
        .features
        .outlook
        .record_outlook_run_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(SavedOutlookRunResponse::from(&run)))
}

/// List the stored zone outlooks, newest issue first
#[utoipa::path(
    get,
    path = "/v1/dashboard/outlooks",
    tag = "outlooks",
    params(OutlookDashboardQuery, PaginationQueryDto),
    responses(
        (status = 200, description = "Outlooks retrieved successfully", body = OutlooksPageResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs outlooks:read", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_outlooks(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<OutlookDashboardQuery>, WebError>,
    WithRejection(Query(page), _): WithRejection<Query<PaginationQueryDto>, WebError>,
) -> Result<ApiResponse<OutlooksPageResponse>, WebError> {
    let pagination = Pagination::from(&page);

    let (outlooks, count) = state
        .features
        .outlook
        .list_zone_outlooks_use_case
        .execute(query.into_input(pagination)?)
        .await?;

    Ok(ApiResponse::ok(OutlooksPageResponse {
        outlooks: outlooks
            .iter()
            .map(|outlook| SavedZoneOutlookResponse::from(outlook).outlook)
            .collect(),
        count,
        page: *pagination.page(),
        rows_per_page: *pagination.rows_per_page(),
    }))
}

/// Add an outlook for a zone, season and issue that has none
#[utoipa::path(
    post,
    path = "/v1/dashboard/outlooks",
    tag = "outlooks",
    request_body = CreateOutlookDashboardParams,
    responses(
        (status = 201, description = "Outlook created", body = SavedZoneOutlookResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs outlooks:create", body = ErrorBody),
        (status = 409, description = "The zone already has an outlook for that season and issue (`already_exists`)", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_create_outlook(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    ValidatedJson(params): ValidatedJson<CreateOutlookDashboardParams>,
) -> Result<ApiResponse<SavedZoneOutlookResponse>, WebError> {
    let outlook = state
        .features
        .outlook
        .create_zone_outlook_use_case
        .execute(&staff_context, params.into_input()?)
        .await?;

    Ok(ApiResponse::created(SavedZoneOutlookResponse::from(
        &outlook,
    )))
}

/// Replace the stored outlook of one zone, season and issue
#[utoipa::path(
    put,
    path = "/v1/dashboard/outlooks/{season}/{issued}/zones/{zone_slug}",
    tag = "outlooks",
    params(
        ("season" = String, Path, description = "Season, for example 2026-27"),
        ("issued" = String, Path, description = "Issue month, YYYY-MM"),
        ("zone_slug" = String, Path, description = "Zone slug, for example chamchamal")
    ),
    request_body = RecordZoneOutlookParams,
    responses(
        (status = 200, description = "Outlook updated", body = SavedZoneOutlookResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs outlooks:update", body = ErrorBody),
        (status = 404, description = "No outlook stored under that key", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_update_outlook(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((season, issued, zone_slug)), _): WithRejection<
        Path<(String, String, String)>,
        WebError,
    >,
    ValidatedJson(params): ValidatedJson<RecordZoneOutlookParams>,
) -> Result<ApiResponse<SavedZoneOutlookResponse>, WebError> {
    let input = params.into_input(season, issued, zone_slug)?;

    let outlook = state
        .features
        .outlook
        .update_zone_outlook_use_case
        .execute(&staff_context, input)
        .await?;

    Ok(ApiResponse::ok(SavedZoneOutlookResponse::from(&outlook)))
}

/// Remove the stored outlook of one zone, season and issue
#[utoipa::path(
    delete,
    path = "/v1/dashboard/outlooks/{season}/{issued}/zones/{zone_slug}",
    tag = "outlooks",
    params(
        ("season" = String, Path, description = "Season, for example 2026-27"),
        ("issued" = String, Path, description = "Issue month, YYYY-MM"),
        ("zone_slug" = String, Path, description = "Zone slug, for example chamchamal")
    ),
    responses(
        (status = 204, description = "Delete was successful, or the outlook was already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs outlooks:delete", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_delete_outlook(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((season, issued, zone_slug)), _): WithRejection<
        Path<(String, String, String)>,
        WebError,
    >,
) -> Result<StatusCode, WebError> {
    let season = Season::new(season).map_err(AppError::from)?;
    let issued = IssueMonth::new(&issued).map_err(AppError::from)?;
    let zone_slug = ZoneSlug::new(zone_slug).map_err(AppError::from)?;

    state
        .features
        .outlook
        .delete_zone_outlook_use_case
        .execute(&staff_context, zone_slug, season, issued)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// List every stored track record, newest issue first
#[utoipa::path(
    get,
    path = "/v1/dashboard/outlook-runs",
    tag = "outlooks",
    responses(
        (status = 200, description = "Track records retrieved successfully", body = OutlookRunsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs outlooks:read", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_outlook_runs(
    State(state): State<AppState>,
) -> Result<ApiResponse<OutlookRunsResponse>, WebError> {
    let runs = state
        .features
        .outlook
        .list_outlook_runs_use_case
        .execute()
        .await?;

    Ok(ApiResponse::ok(OutlookRunsResponse {
        runs: runs
            .iter()
            .map(|run| SavedOutlookRunResponse::from(run).run)
            .collect(),
    }))
}

/// Add a track record for a season and issue that has none
#[utoipa::path(
    post,
    path = "/v1/dashboard/outlook-runs",
    tag = "outlooks",
    request_body = CreateOutlookRunDashboardParams,
    responses(
        (status = 201, description = "Track record created", body = SavedOutlookRunResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs outlooks:create", body = ErrorBody),
        (status = 409, description = "The season and issue already have a track record (`already_exists`)", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_create_outlook_run(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    ValidatedJson(params): ValidatedJson<CreateOutlookRunDashboardParams>,
) -> Result<ApiResponse<SavedOutlookRunResponse>, WebError> {
    let run = state
        .features
        .outlook
        .create_outlook_run_use_case
        .execute(&staff_context, params.into_input()?)
        .await?;

    Ok(ApiResponse::created(SavedOutlookRunResponse::from(&run)))
}

/// Replace the stored track record of one season and issue
#[utoipa::path(
    put,
    path = "/v1/dashboard/outlook-runs/{season}/{issued}",
    tag = "outlooks",
    params(
        ("season" = String, Path, description = "Season, for example 2026-27"),
        ("issued" = String, Path, description = "Issue month, YYYY-MM")
    ),
    request_body = RecordOutlookRunParams,
    responses(
        (status = 200, description = "Track record updated", body = SavedOutlookRunResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs outlooks:update", body = ErrorBody),
        (status = 404, description = "No track record stored under that key", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_update_outlook_run(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((season, issued)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<RecordOutlookRunParams>,
) -> Result<ApiResponse<SavedOutlookRunResponse>, WebError> {
    let input = params.into_input(season, issued)?;

    let run = state
        .features
        .outlook
        .update_outlook_run_use_case
        .execute(&staff_context, input)
        .await?;

    Ok(ApiResponse::ok(SavedOutlookRunResponse::from(&run)))
}

/// Remove the stored track record of one season and issue
#[utoipa::path(
    delete,
    path = "/v1/dashboard/outlook-runs/{season}/{issued}",
    tag = "outlooks",
    params(
        ("season" = String, Path, description = "Season, for example 2026-27"),
        ("issued" = String, Path, description = "Issue month, YYYY-MM")
    ),
    responses(
        (status = 204, description = "Delete was successful, or the track record was already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs outlooks:delete", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_delete_outlook_run(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((season, issued)), _): WithRejection<Path<(String, String)>, WebError>,
) -> Result<StatusCode, WebError> {
    let season = Season::new(season).map_err(AppError::from)?;
    let issued = IssueMonth::new(&issued).map_err(AppError::from)?;

    state
        .features
        .outlook
        .delete_outlook_run_use_case
        .execute(&staff_context, season, issued)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
