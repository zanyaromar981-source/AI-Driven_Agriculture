use axum::{
    Extension,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        CreateFarmParams, FarmStatusResponse, FarmSummaryResponse, FarmsResponse, OneFarmResponse,
        RepaintFarmCellsParams, SavedFarmResponse,
    },
    errors::WebError,
};

use crate::{
    app::AuthContext,
    infra::http::{ApiResponse, ErrorBody, ValidatedJson},
    shared::AppState,
};

const IDEMPOTENCY_KEY: &str = "idempotency-key";

/// Farm ids travel as opaque strings. One that is not a number cannot name
/// a farm, so it is not found rather than a bad request.
fn farm_id(raw: &str) -> Result<i32, WebError> {
    raw.parse().map_err(|_| WebError::not_found())
}

/// List all farms for the authenticated user
#[utoipa::path(
    get,
    path = "/v1/farms",
    tag = "farms",
    responses(
        (status = 200, description = "List of farms retrieved successfully", body = FarmsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_farms(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
) -> Result<ApiResponse<FarmsResponse>, WebError> {
    let farms = state
        .features
        .farm
        .list_farms_use_case
        .execute(&auth_context)
        .await?;

    Ok(ApiResponse::ok(FarmsResponse {
        farms: farms.iter().map(FarmSummaryResponse::from).collect(),
    }))
}

/// Create a new farm from a walked outline and its painted cells
#[utoipa::path(
    post,
    path = "/v1/farms",
    tag = "farms",
    request_body = CreateFarmParams,
    params(("Idempotency-Key" = Option<String>, Header, description = "Repeat it on a retry to get the farm already created")),
    responses(
        (status = 201, description = "Farm created successfully", body = SavedFarmResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_farm(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    headers: HeaderMap,
    ValidatedJson(params): ValidatedJson<CreateFarmParams>,
) -> Result<ApiResponse<SavedFarmResponse>, WebError> {
    let idempotency_key = headers
        .get(IDEMPOTENCY_KEY)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    let input = params.into_input(idempotency_key)?;

    let (farm, dropped_cells) = state
        .features
        .farm
        .register_farm_use_case
        .execute(&auth_context, input)
        .await?;

    Ok(ApiResponse::created(SavedFarmResponse::try_from((
        &farm,
        dropped_cells.as_slice(),
    ))?))
}

/// Get one farm with its outline and cells
#[utoipa::path(
    get,
    path = "/v1/farms/{id}",
    tag = "farms",
    params(("id" = String, Path, description = "Farm ID")),
    responses(
        (status = 200, description = "Farm retrieved successfully", body = OneFarmResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_farm(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<ApiResponse<OneFarmResponse>, WebError> {
    let farm = state
        .features
        .farm
        .view_farm_use_case
        .execute(&auth_context, farm_id(&id)?)
        .await?;

    Ok(ApiResponse::ok(OneFarmResponse::try_from(&farm)?))
}

/// Get a farm's status from space
///
/// A placeholder so the app's Home opens: there is no store for satellite
/// readings yet, so the measured fields are always `null`. The crop plots
/// are real and come from the farm itself.
#[utoipa::path(
    get,
    path = "/v1/farms/{id}/status",
    tag = "farms",
    params(("id" = String, Path, description = "Farm ID")),
    responses(
        (status = 200, description = "Farm status retrieved successfully", body = FarmStatusResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_farm_status(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<ApiResponse<FarmStatusResponse>, WebError> {
    let farm = state
        .features
        .farm
        .view_farm_use_case
        .execute(&auth_context, farm_id(&id)?)
        .await?;

    Ok(ApiResponse::ok(FarmStatusResponse::from(&farm)))
}

/// Repaint the crops on a farm's cells
#[utoipa::path(
    put,
    path = "/v1/farms/{id}/cells",
    tag = "farms",
    params(("id" = String, Path, description = "Farm ID")),
    request_body = RepaintFarmCellsParams,
    responses(
        (status = 200, description = "Farm cells repainted successfully", body = SavedFarmResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn repaint_farm_cells(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<RepaintFarmCellsParams>,
) -> Result<ApiResponse<SavedFarmResponse>, WebError> {
    let input = params.into_input();

    let (farm, dropped_cells) = state
        .features
        .farm
        .repaint_farm_cells_use_case
        .execute(&auth_context, farm_id(&id)?, input)
        .await?;

    Ok(ApiResponse::ok(SavedFarmResponse::try_from((
        &farm,
        dropped_cells.as_slice(),
    ))?))
}

/// Delete a specific farm
#[utoipa::path(
    delete,
    path = "/v1/farms/{id}",
    tag = "farms",
    params(("id" = String, Path, description = "Farm ID")),
    responses(
        (status = 204, description = "Delete was successful"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_farm(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<StatusCode, WebError> {
    state
        .features
        .farm
        .remove_farm_use_case
        .execute(&auth_context, farm_id(&id)?)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
