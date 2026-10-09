use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        FireDashboardCreateParams, FireDashboardListResponse, FireDashboardOneResponse,
        FireDashboardQuery, FireDashboardResponse, FireResponse, FiresQueryDto, FiresResponse,
        RecordFireParams,
    },
    errors::WebError,
};

use crate::{
    app::{Pagination, StaffContext},
    infra::http::{ApiResponse, ErrorBody, PaginationQueryDto, ValidatedJson},
    shared::AppState,
};

/// Fire ids travel as opaque strings. One that is not a number cannot name
/// a fire, so it is not found rather than a bad request.
fn fire_id(raw: &str) -> Result<i32, WebError> {
    raw.parse().map_err(|_| WebError::not_found())
}

/// List the fires detected in the last hours, with their totals
#[utoipa::path(
    get,
    path = "/v1/fires",
    tag = "fires",
    params(FiresQueryDto),
    responses(
        (status = 200, description = "Fires retrieved successfully", body = FiresResponse),
        (status = 400, description = "Invalid query", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_fires(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<FiresQueryDto>, WebError>,
) -> Result<ApiResponse<FiresResponse>, WebError> {
    let window = query.into_window()?;

    let (fires, summary) = state
        .features
        .fire
        .list_fires_use_case
        .execute(window)
        .await?;

    Ok(ApiResponse::ok(FiresResponse::try_from((
        window,
        fires.as_slice(),
        &summary,
    ))?))
}

/// Store one fire under the job's own id, replacing an earlier push of it
#[utoipa::path(
    put,
    path = "/v1/ingest/fires/{external_id}",
    tag = "fires",
    params(
        ("external_id" = String, Path, description = "The job's own key for the fire, 1 to 100 printable ASCII characters"),
        ("X-Service-Key" = String, Header, description = "The ingest service key")
    ),
    request_body = RecordFireParams,
    responses(
        (status = 200, description = "Fire stored", body = FireResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_fire(
    State(state): State<AppState>,
    WithRejection(Path(external_id), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<RecordFireParams>,
) -> Result<ApiResponse<FireResponse>, WebError> {
    let input = params.into_input(external_id)?;

    let fire = state
        .features
        .fire
        .record_fire_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(FireResponse::try_from(&fire)?))
}

/// List the stored fires for the editing screen, newest first
#[utoipa::path(
    get,
    path = "/v1/dashboard/fires",
    tag = "fires",
    params(FireDashboardQuery, PaginationQueryDto),
    responses(
        (status = 200, description = "Fires retrieved successfully", body = FireDashboardListResponse),
        (status = 400, description = "Invalid query", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs fires:read", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard_fires(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<FireDashboardQuery>, WebError>,
    WithRejection(Query(page), _): WithRejection<Query<PaginationQueryDto>, WebError>,
) -> Result<ApiResponse<FireDashboardListResponse>, WebError> {
    let pagination = Pagination::from(&page);
    let input = query.into_input(pagination)?;

    let (fires, count) = state
        .features
        .fire
        .list_stored_fires_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(FireDashboardListResponse {
        fires: fires
            .iter()
            .map(FireDashboardResponse::try_from)
            .collect::<Result<Vec<_>, _>>()?,
        count,
        page: *pagination.page(),
        rows_per_page: *pagination.rows_per_page(),
    }))
}

/// Get one stored fire
#[utoipa::path(
    get,
    path = "/v1/dashboard/fires/{id}",
    tag = "fires",
    params(("id" = String, Path, description = "Fire ID")),
    responses(
        (status = 200, description = "Fire retrieved successfully", body = FireDashboardOneResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs fires:read", body = ErrorBody),
        (status = 404, description = "Fire not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard_fire(
    State(state): State<AppState>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<ApiResponse<FireDashboardOneResponse>, WebError> {
    let fire = state
        .features
        .fire
        .view_stored_fire_use_case
        .execute(fire_id(&id)?)
        .await?;

    Ok(ApiResponse::ok(FireDashboardOneResponse {
        fire: FireDashboardResponse::try_from(&fire)?,
    }))
}

/// Enter a fire by hand under a new external id
#[utoipa::path(
    post,
    path = "/v1/dashboard/fires",
    tag = "fires",
    request_body = FireDashboardCreateParams,
    responses(
        (status = 201, description = "Fire created successfully", body = FireDashboardOneResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs fires:create", body = ErrorBody),
        (status = 409, description = "A fire is already stored under this external id (`already_exists`)", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_dashboard_fire(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    ValidatedJson(params): ValidatedJson<FireDashboardCreateParams>,
) -> Result<ApiResponse<FireDashboardOneResponse>, WebError> {
    let input = params.into_input()?;

    let fire = state
        .features
        .fire
        .create_fire_use_case
        .execute(&staff_context, input)
        .await?;

    Ok(ApiResponse::created(FireDashboardOneResponse {
        fire: FireDashboardResponse::try_from(&fire)?,
    }))
}

/// Replace every field of a stored fire except its external id
#[utoipa::path(
    put,
    path = "/v1/dashboard/fires/{id}",
    tag = "fires",
    params(("id" = String, Path, description = "Fire ID")),
    request_body = RecordFireParams,
    responses(
        (status = 200, description = "Fire updated successfully", body = FireDashboardOneResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs fires:update", body = ErrorBody),
        (status = 404, description = "Fire not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_dashboard_fire(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<RecordFireParams>,
) -> Result<ApiResponse<FireDashboardOneResponse>, WebError> {
    let input = params.into_correction_input()?;

    let fire = state
        .features
        .fire
        .correct_fire_use_case
        .execute(&staff_context, fire_id(&id)?, input)
        .await?;

    Ok(ApiResponse::ok(FireDashboardOneResponse {
        fire: FireDashboardResponse::try_from(&fire)?,
    }))
}

/// Remove a stored fire
#[utoipa::path(
    delete,
    path = "/v1/dashboard/fires/{id}",
    tag = "fires",
    params(("id" = String, Path, description = "Fire ID")),
    responses(
        (status = 204, description = "Delete was successful, also when the fire was already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs fires:delete", body = ErrorBody),
        (status = 404, description = "The fire id is not a number", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_dashboard_fire(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<StatusCode, WebError> {
    state
        .features
        .fire
        .remove_fire_use_case
        .execute(&staff_context, fire_id(&id)?)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
