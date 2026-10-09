use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        CreateDamDashboardReadingParams, DamDashboardReadingsQuery, DamHistoryQuery,
        DamHistoryResponse, DamReadingResponse, DamReadingsPageResponse, DamReferenceResponse,
        DamReferencesResponse, DamResponse, DamsResponse, RecordDamReadingParams,
        SavedDamReadingResponse, parse_day,
    },
    errors::WebError,
};

use crate::{
    app::{Pagination, StaffContext},
    features::dams::domain::DamSlug,
    infra::http::{ApiResponse, ErrorBody, PaginationQueryDto, ValidatedJson},
    shared::AppState,
};

/// A slug that is not well formed cannot name a dam, so on a read it is not
/// found rather than a bad request.
fn dam_slug(raw: String) -> Result<DamSlug, WebError> {
    DamSlug::new(raw).map_err(|_| WebError::not_found())
}

/// List the dams with their latest reading and the one from a year before
#[utoipa::path(
    get,
    path = "/v1/dams",
    tag = "dams",
    responses(
        (status = 200, description = "Dams retrieved successfully", body = DamsResponse),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_dams(
    State(state): State<AppState>,
) -> Result<ApiResponse<DamsResponse>, WebError> {
    let statuses = state.features.dam.list_dams_use_case.execute().await?;

    Ok(ApiResponse::ok(DamsResponse {
        dams: statuses.iter().map(DamResponse::from).collect(),
    }))
}

/// Get one dam's readings over time, oldest first
#[utoipa::path(
    get,
    path = "/v1/dams/{slug}/history",
    tag = "dams",
    params(("slug" = String, Path, description = "Dam slug, for example dukan"), DamHistoryQuery),
    responses(
        (status = 200, description = "History retrieved successfully", body = DamHistoryResponse),
        (status = 404, description = "Dam not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_dam_history(
    State(state): State<AppState>,
    WithRejection(Path(slug), _): WithRejection<Path<String>, WebError>,
    WithRejection(Query(query), _): WithRejection<Query<DamHistoryQuery>, WebError>,
) -> Result<ApiResponse<DamHistoryResponse>, WebError> {
    let input = query.into_input(dam_slug(slug)?)?;

    let (dam, readings) = state
        .features
        .dam
        .view_dam_history_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(DamHistoryResponse::from((
        &dam,
        readings.as_slice(),
    ))))
}

/// Store one dam's reading for one day, replacing an earlier one for that day
#[utoipa::path(
    put,
    path = "/v1/ingest/dams/{slug}/readings/{day}",
    tag = "dams",
    params(
        ("slug" = String, Path, description = "Dam slug, for example dukan"),
        ("day" = String, Path, description = "Day of the reading, YYYY-MM-DD"),
        ("X-Service-Key" = String, Header, description = "The data jobs' shared key")
    ),
    request_body = RecordDamReadingParams,
    responses(
        (status = 200, description = "Reading stored", body = SavedDamReadingResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 404, description = "Dam not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_dam_reading(
    State(state): State<AppState>,
    WithRejection(Path((slug, day)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<RecordDamReadingParams>,
) -> Result<ApiResponse<SavedDamReadingResponse>, WebError> {
    let input = params.into_input(slug.clone(), day)?;

    let reading = state
        .features
        .dam
        .record_dam_reading_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(SavedDamReadingResponse {
        slug,
        reading: (&reading).into(),
    }))
}

/// List the dams as reference data, for the dashboard's editing screens
#[utoipa::path(
    get,
    path = "/v1/dashboard/dams",
    tag = "dams",
    responses(
        (status = 200, description = "Dams retrieved successfully", body = DamReferencesResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs dams:read", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_dams(
    State(state): State<AppState>,
) -> Result<ApiResponse<DamReferencesResponse>, WebError> {
    let dams = state
        .features
        .dam
        .list_reference_dams_use_case
        .execute()
        .await?;

    Ok(ApiResponse::ok(DamReferencesResponse {
        dams: dams.iter().map(DamReferenceResponse::from).collect(),
    }))
}

/// List one dam's stored readings, newest day first
#[utoipa::path(
    get,
    path = "/v1/dashboard/dams/{slug}/readings",
    tag = "dams",
    params(
        ("slug" = String, Path, description = "Dam slug, for example dukan"),
        DamDashboardReadingsQuery,
        PaginationQueryDto
    ),
    responses(
        (status = 200, description = "Readings retrieved successfully", body = DamReadingsPageResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs dams:read", body = ErrorBody),
        (status = 404, description = "Dam not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_dam_readings(
    State(state): State<AppState>,
    WithRejection(Path(slug), _): WithRejection<Path<String>, WebError>,
    WithRejection(Query(query), _): WithRejection<Query<DamDashboardReadingsQuery>, WebError>,
    WithRejection(Query(page), _): WithRejection<Query<PaginationQueryDto>, WebError>,
) -> Result<ApiResponse<DamReadingsPageResponse>, WebError> {
    let pagination = Pagination::from(&page);
    let input = query.into_input(dam_slug(slug)?, pagination)?;

    let (readings, count) = state
        .features
        .dam
        .list_dam_readings_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(DamReadingsPageResponse {
        readings: readings.iter().map(DamReadingResponse::from).collect(),
        count,
        page: *pagination.page(),
        rows_per_page: *pagination.rows_per_page(),
    }))
}

/// Add a reading for a day the dam has none for
#[utoipa::path(
    post,
    path = "/v1/dashboard/dams/{slug}/readings",
    tag = "dams",
    params(("slug" = String, Path, description = "Dam slug, for example dukan")),
    request_body = CreateDamDashboardReadingParams,
    responses(
        (status = 201, description = "Reading created", body = SavedDamReadingResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs dams:create", body = ErrorBody),
        (status = 404, description = "Dam not found", body = ErrorBody),
        (status = 409, description = "The dam already has a reading for that day (`already_exists`)", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_create_dam_reading(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(slug), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<CreateDamDashboardReadingParams>,
) -> Result<ApiResponse<SavedDamReadingResponse>, WebError> {
    let slug = String::from(&dam_slug(slug)?);
    let input = params.into_input(slug.clone())?;

    let reading = state
        .features
        .dam
        .create_dam_reading_use_case
        .execute(&staff_context, input)
        .await?;

    Ok(ApiResponse::created(SavedDamReadingResponse {
        slug,
        reading: (&reading).into(),
    }))
}

/// Replace the stored reading of one dam and day
#[utoipa::path(
    put,
    path = "/v1/dashboard/dams/{slug}/readings/{day}",
    tag = "dams",
    params(
        ("slug" = String, Path, description = "Dam slug, for example dukan"),
        ("day" = String, Path, description = "Day of the reading, YYYY-MM-DD")
    ),
    request_body = RecordDamReadingParams,
    responses(
        (status = 200, description = "Reading updated", body = SavedDamReadingResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs dams:update", body = ErrorBody),
        (status = 404, description = "Dam not found, or no reading for that day", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_update_dam_reading(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((slug, day)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<RecordDamReadingParams>,
) -> Result<ApiResponse<SavedDamReadingResponse>, WebError> {
    let slug = String::from(&dam_slug(slug)?);
    let input = params.into_input(slug.clone(), day)?;

    let reading = state
        .features
        .dam
        .update_dam_reading_use_case
        .execute(&staff_context, input)
        .await?;

    Ok(ApiResponse::ok(SavedDamReadingResponse {
        slug,
        reading: (&reading).into(),
    }))
}

/// Remove the stored reading of one dam and day
#[utoipa::path(
    delete,
    path = "/v1/dashboard/dams/{slug}/readings/{day}",
    tag = "dams",
    params(
        ("slug" = String, Path, description = "Dam slug, for example dukan"),
        ("day" = String, Path, description = "Day of the reading, YYYY-MM-DD")
    ),
    responses(
        (status = 204, description = "Delete was successful, or the reading was already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs dams:delete", body = ErrorBody),
        (status = 404, description = "Dam not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_delete_dam_reading(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((slug, day)), _): WithRejection<Path<(String, String)>, WebError>,
) -> Result<StatusCode, WebError> {
    let slug = dam_slug(slug)?;
    let day = parse_day("day", &day)?;

    state
        .features
        .dam
        .delete_dam_reading_use_case
        .execute(&staff_context, slug, day)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
