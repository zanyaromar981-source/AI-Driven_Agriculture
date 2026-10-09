use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        BriefDashboardListQuery, BriefFarmZonesRecordedResponse, BriefLatestQuery, BriefListQuery,
        BriefResponse, BriefsPageResponse, BriefsResponse, FarmBriefResponse, OneBriefResponse,
        RecordBriefFarmZonesParams, RecordBriefParams, parse_day, scope,
    },
    errors::WebError,
};

use crate::{
    app::{AuthContext, Pagination, StaffContext},
    infra::http::{ApiResponse, ErrorBody, PaginationQueryDto, ValidatedJson},
    shared::AppState,
};

/// Farm ids travel as opaque strings. One that is not a number cannot name
/// a farm, so it is not found rather than a bad request.
fn farm_id(raw: &str) -> Result<i32, WebError> {
    raw.parse().map_err(|_| WebError::not_found())
}

/// Get the newest brief of the region or of one zone
#[utoipa::path(
    get,
    path = "/v1/briefs/latest",
    tag = "briefs",
    params(BriefLatestQuery),
    responses(
        (status = 200, description = "Brief retrieved successfully", body = OneBriefResponse),
        (status = 404, description = "No brief is stored for this scope", body = ErrorBody),
        (status = 422, description = "The scope is not `region` or a zone slug", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_latest_brief(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<BriefLatestQuery>, WebError>,
) -> Result<ApiResponse<OneBriefResponse>, WebError> {
    let brief = state
        .features
        .brief
        .view_latest_brief_use_case
        .execute(&query.into_scope()?)
        .await?;

    Ok(ApiResponse::ok(OneBriefResponse::from(&brief)))
}

/// List the briefs of the region or of one zone, newest day first
#[utoipa::path(
    get,
    path = "/v1/briefs",
    tag = "briefs",
    params(BriefListQuery),
    responses(
        (status = 200, description = "Briefs retrieved successfully", body = BriefsResponse),
        (status = 422, description = "Validation error: `bad_range` when `from` is after `to` or the range is longer than 92 days", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_briefs(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<BriefListQuery>, WebError>,
) -> Result<ApiResponse<BriefsResponse>, WebError> {
    let briefs = state
        .features
        .brief
        .list_briefs_use_case
        .execute(query.into_input()?)
        .await?;

    Ok(ApiResponse::ok(BriefsResponse {
        briefs: briefs.iter().map(BriefResponse::from).collect(),
    }))
}

/// Get the brief for one of the farmer's own farms
#[utoipa::path(
    get,
    path = "/v1/farms/{id}/brief",
    tag = "briefs",
    params(("id" = String, Path, description = "Farm ID")),
    responses(
        (status = 200, description = "The brief of the farm's zone, or of the region, or null when none is stored yet", body = FarmBriefResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_farm_brief(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<ApiResponse<FarmBriefResponse>, WebError> {
    let farm_brief = state
        .features
        .brief
        .view_farm_brief_use_case
        .execute(&auth_context, farm_id(&id)?)
        .await?;

    Ok(ApiResponse::ok(FarmBriefResponse::from(&farm_brief)))
}

/// Store the brief of one day for one scope, replacing the one it had
#[utoipa::path(
    put,
    path = "/v1/ingest/briefs/{day}/{scope}",
    tag = "briefs",
    params(
        ("day" = String, Path, description = "The day the brief is for, `YYYY-MM-DD`"),
        ("scope" = String, Path, description = "`region` or a zone slug such as `chamchamal`"),
        ("X-Service-Key" = String, Header, description = "The ingest service key")
    ),
    request_body = RecordBriefParams,
    responses(
        (status = 200, description = "Brief stored", body = OneBriefResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 422, description = "Validation error: `invalid`, `bad_points` or `bad_sources`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_brief(
    State(state): State<AppState>,
    WithRejection(Path((day, raw_scope)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<RecordBriefParams>,
) -> Result<ApiResponse<OneBriefResponse>, WebError> {
    let input = params.into_input(&day, raw_scope)?;

    let brief = state
        .features
        .brief
        .record_brief_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(OneBriefResponse::from(&brief)))
}

/// Record which zone each farm lies in
#[utoipa::path(
    put,
    path = "/v1/ingest/briefs/farm-zones",
    tag = "briefs",
    params(("X-Service-Key" = String, Header, description = "The ingest service key")),
    request_body = RecordBriefFarmZonesParams,
    responses(
        (status = 200, description = "Every farm's zone recorded", body = BriefFarmZonesRecordedResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 422, description = "Validation error: `invalid`, `bad_farms` or `duplicate_farm`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_farm_zones(
    State(state): State<AppState>,
    ValidatedJson(params): ValidatedJson<RecordBriefFarmZonesParams>,
) -> Result<ApiResponse<BriefFarmZonesRecordedResponse>, WebError> {
    let recorded = state
        .features
        .brief
        .record_farm_zones_use_case
        .execute(params.into_farm_zones()?)
        .await?;

    Ok(ApiResponse::ok(BriefFarmZonesRecordedResponse { recorded }))
}

/// Remove the brief of one day for one scope
#[utoipa::path(
    delete,
    path = "/v1/ingest/briefs/{day}/{scope}",
    tag = "briefs",
    params(
        ("day" = String, Path, description = "The day the brief is for, `YYYY-MM-DD`"),
        ("scope" = String, Path, description = "`region` or a zone slug such as `chamchamal`"),
        ("X-Service-Key" = String, Header, description = "The ingest service key")
    ),
    responses(
        (status = 204, description = "Delete was successful, also when the brief was already gone"),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 422, description = "The day or the scope is not valid", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn delete_brief(
    State(state): State<AppState>,
    WithRejection(Path((day, raw_scope)), _): WithRejection<Path<(String, String)>, WebError>,
) -> Result<StatusCode, WebError> {
    state
        .features
        .brief
        .delete_brief_use_case
        .execute(parse_day("day", &day)?, &scope(raw_scope)?)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// List the stored briefs of every scope, newest day first
#[utoipa::path(
    get,
    path = "/v1/dashboard/briefs",
    tag = "briefs",
    params(BriefDashboardListQuery, PaginationQueryDto),
    responses(
        (status = 200, description = "Briefs retrieved successfully", body = BriefsPageResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs briefs:read", body = ErrorBody),
        (status = 422, description = "Validation error: `bad_range` when `from` is after `to`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard_briefs(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<BriefDashboardListQuery>, WebError>,
    WithRejection(Query(page), _): WithRejection<Query<PaginationQueryDto>, WebError>,
) -> Result<ApiResponse<BriefsPageResponse>, WebError> {
    let pagination = Pagination::from(&page);

    let (briefs, count) = state
        .features
        .brief
        .list_stored_briefs_use_case
        .execute(query.into_input(pagination)?)
        .await?;

    Ok(ApiResponse::ok(BriefsPageResponse {
        briefs: briefs.iter().map(BriefResponse::from).collect(),
        count,
        page: *pagination.page(),
        rows_per_page: *pagination.rows_per_page(),
    }))
}

/// Replace the brief stored for one day and scope
#[utoipa::path(
    put,
    path = "/v1/dashboard/briefs/{day}/{scope}",
    tag = "briefs",
    params(
        ("day" = String, Path, description = "The day the brief is for, `YYYY-MM-DD`"),
        ("scope" = String, Path, description = "`region` or a zone slug such as `chamchamal`")
    ),
    request_body = RecordBriefParams,
    responses(
        (status = 200, description = "Brief updated successfully", body = OneBriefResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs briefs:update", body = ErrorBody),
        (status = 404, description = "No brief is stored for this day and scope", body = ErrorBody),
        (status = 422, description = "Validation error: `invalid`, `bad_points` or `bad_sources`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_dashboard_brief(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((day, raw_scope)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<RecordBriefParams>,
) -> Result<ApiResponse<OneBriefResponse>, WebError> {
    let input = params.into_input(&day, raw_scope)?;

    let brief = state
        .features
        .brief
        .correct_brief_use_case
        .execute(&staff_context, input)
        .await?;

    Ok(ApiResponse::ok(OneBriefResponse::from(&brief)))
}

/// Remove the brief stored for one day and scope
#[utoipa::path(
    delete,
    path = "/v1/dashboard/briefs/{day}/{scope}",
    tag = "briefs",
    params(
        ("day" = String, Path, description = "The day the brief is for, `YYYY-MM-DD`"),
        ("scope" = String, Path, description = "`region` or a zone slug such as `chamchamal`")
    ),
    responses(
        (status = 204, description = "Delete was successful, also when the brief was already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs briefs:delete", body = ErrorBody),
        (status = 422, description = "The day or the scope is not valid", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_dashboard_brief(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((day, raw_scope)), _): WithRejection<Path<(String, String)>, WebError>,
) -> Result<StatusCode, WebError> {
    state
        .features
        .brief
        .remove_brief_use_case
        .execute(&staff_context, parse_day("day", &day)?, &scope(raw_scope)?)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
