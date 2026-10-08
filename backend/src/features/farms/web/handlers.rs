use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{CreateFarmParams, FarmSummaryResponse, RepaintFarmCellsParams, SavedFarmResponse},
    errors::WebError,
};

use crate::{
    app::{AuthContext, Pagination},
    infra::http::{ErrorWrapper, Meta, PaginationQueryDto, ResponseWrapper, ValidatedJson},
    shared::AppState,
};

/// List all farms for the authenticated user
#[utoipa::path(
    get,
    path = "/v1/farms",
    tag = "farms",
    params(PaginationQueryDto),
    responses(
        (status = 200, description = "List of farms retrieved successfully", body = ResponseWrapper<Vec<FarmSummaryResponse>>),
        (status = 400, description = "Invalid query parameter", body = ErrorWrapper),
        (status = 401, description = "Unauthorized", body = ErrorWrapper),
        (status = 500, description = "Internal server error", body = ErrorWrapper)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_farms(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Query(query), _): WithRejection<Query<PaginationQueryDto>, WebError>,
) -> Result<ResponseWrapper<Vec<FarmSummaryResponse>>, WebError> {
    let pagination = Pagination::from(&query);

    let (farms, total_count) = state
        .features
        .farm
        .list_farms_use_case
        .execute(&auth_context, &pagination)
        .await?;

    let data = farms.iter().map(FarmSummaryResponse::from).collect();

    let meta = Meta {
        count: total_count,
        page: Some(*pagination.page()),
        rows_per_page: Some(*pagination.rows_per_page()),
    };

    Ok(ResponseWrapper::new(data, Some(meta), StatusCode::OK))
}

/// Create a new farm from a walked outline and its painted cells
#[utoipa::path(
    post,
    path = "/v1/farms",
    tag = "farms",
    request_body = CreateFarmParams,
    responses(
        (status = 201, description = "Farm created successfully", body = ResponseWrapper<SavedFarmResponse>),
        (status = 400, description = "Invalid request body", body = ErrorWrapper),
        (status = 401, description = "Unauthorized", body = ErrorWrapper),
        (status = 422, description = "Validation error", body = ErrorWrapper),
        (status = 500, description = "Internal server error", body = ErrorWrapper)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_farm(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    ValidatedJson(params): ValidatedJson<CreateFarmParams>,
) -> Result<ResponseWrapper<SavedFarmResponse>, WebError> {
    let input = params.into_input()?;

    let (farm, dropped_cells) = state
        .features
        .farm
        .register_farm_use_case
        .execute(&auth_context, input)
        .await?;

    Ok(ResponseWrapper::new(
        SavedFarmResponse::try_from((&farm, dropped_cells.as_slice()))?,
        None,
        StatusCode::CREATED,
    ))
}

/// Repaint the crops on a farm's cells
#[utoipa::path(
    put,
    path = "/v1/farms/{id}/cells",
    tag = "farms",
    params(("id" = i32, Path, description = "Farm ID")),
    request_body = RepaintFarmCellsParams,
    responses(
        (status = 200, description = "Farm cells repainted successfully", body = ResponseWrapper<SavedFarmResponse>),
        (status = 400, description = "Invalid request body", body = ErrorWrapper),
        (status = 401, description = "Unauthorized", body = ErrorWrapper),
        (status = 404, description = "Farm not found", body = ErrorWrapper),
        (status = 422, description = "Validation error", body = ErrorWrapper),
        (status = 500, description = "Internal server error", body = ErrorWrapper)
    ),
    security(("bearer_auth" = []))
)]
pub async fn repaint_farm_cells(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path(id), _): WithRejection<Path<i32>, WebError>,
    ValidatedJson(params): ValidatedJson<RepaintFarmCellsParams>,
) -> Result<ResponseWrapper<SavedFarmResponse>, WebError> {
    let input = params.into_input();

    let (farm, dropped_cells) = state
        .features
        .farm
        .repaint_farm_cells_use_case
        .execute(&auth_context, id, input)
        .await?;

    Ok(ResponseWrapper::new(
        SavedFarmResponse::try_from((&farm, dropped_cells.as_slice()))?,
        None,
        StatusCode::OK,
    ))
}

/// Delete a specific farm
#[utoipa::path(
    delete,
    path = "/v1/farms/{id}",
    tag = "farms",
    params(("id" = i32, Path, description = "Farm ID")),
    responses(
        (status = 204, description = "Delete was successful"),
        (status = 401, description = "Unauthorized", body = ErrorWrapper),
        (status = 404, description = "Farm not found", body = ErrorWrapper),
        (status = 500, description = "Internal server error", body = ErrorWrapper)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_farm(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path(id), _): WithRejection<Path<i32>, WebError>,
) -> Result<StatusCode, WebError> {
    state
        .features
        .farm
        .remove_farm_use_case
        .execute(&auth_context, id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
