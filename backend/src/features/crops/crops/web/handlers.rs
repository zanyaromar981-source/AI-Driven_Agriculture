use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;
use chrono::Utc;

use super::{
    dtos::{CreateCropParams, CropResponse, CropsResponse, OneCropResponse, UpdateCropParams},
    errors::WebError,
};

use crate::{
    app::StaffContext,
    features::crops::domain::CropCode,
    infra::http::{ApiResponse, ErrorBody, ValidatedJson},
    shared::AppState,
};

async fn crops(state: &AppState, only_active: bool) -> Result<CropsResponse, WebError> {
    let crops = state
        .features
        .crop
        .list_crops_use_case
        .execute(only_active)
        .await?;

    Ok(CropsResponse {
        crops: crops.iter().map(CropResponse::from).collect(),
    })
}

/// List the crops that are switched on, with their names and colours
#[utoipa::path(
    get,
    path = "/v1/crops",
    tag = "crops",
    responses(
        (status = 200, description = "Crops retrieved successfully", body = CropsResponse),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_crops(
    State(state): State<AppState>,
) -> Result<ApiResponse<CropsResponse>, WebError> {
    Ok(ApiResponse::ok(crops(&state, true).await?))
}

/// List every crop as stored, also the ones switched off
#[utoipa::path(
    get,
    path = "/v1/dashboard/crops",
    tag = "crops",
    responses(
        (status = 200, description = "Crops retrieved successfully", body = CropsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs crops:read", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_crops(
    State(state): State<AppState>,
) -> Result<ApiResponse<CropsResponse>, WebError> {
    Ok(ApiResponse::ok(crops(&state, false).await?))
}

/// Add a crop
#[utoipa::path(
    post,
    path = "/v1/dashboard/crops",
    tag = "crops",
    request_body = CreateCropParams,
    responses(
        (status = 201, description = "Crop created successfully", body = OneCropResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs crops:create", body = ErrorBody),
        (status = 409, description = "A crop has this code (`already_exists`)", body = ErrorBody),
        (status = 422, description = "Validation error, or the code `empty` (`reserved_code`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_create_crop(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    ValidatedJson(params): ValidatedJson<CreateCropParams>,
) -> Result<ApiResponse<OneCropResponse>, WebError> {
    let input = params.into_input()?;

    let crop = state
        .features
        .crop
        .create_crop_use_case
        .execute(&staff_context, input, Utc::now())
        .await?;

    Ok(ApiResponse::created(OneCropResponse::from(&crop)))
}

/// Replace everything but the code of a crop
#[utoipa::path(
    put,
    path = "/v1/dashboard/crops/{code}",
    tag = "crops",
    params(("code" = String, Path, description = "Crop code, for example wheat")),
    request_body = UpdateCropParams,
    responses(
        (status = 200, description = "Crop updated successfully", body = OneCropResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs crops:update", body = ErrorBody),
        (status = 404, description = "Crop not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_update_crop(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(code), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<UpdateCropParams>,
) -> Result<ApiResponse<OneCropResponse>, WebError> {
    // A code that is not well formed cannot name a crop, so it is not found
    // rather than a bad request.
    let code = CropCode::new(code).map_err(|_| WebError::not_found())?;
    let details = params.into_input()?;

    let crop = state
        .features
        .crop
        .update_crop_use_case
        .execute(&staff_context, code, details, Utc::now())
        .await?;

    Ok(ApiResponse::ok(OneCropResponse::from(&crop)))
}

/// Remove a crop nothing uses
#[utoipa::path(
    delete,
    path = "/v1/dashboard/crops/{code}",
    tag = "crops",
    params(("code" = String, Path, description = "Crop code, for example wheat")),
    responses(
        (status = 204, description = "Crop removed, or it was already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs crops:delete", body = ErrorBody),
        (status = 409, description = "A farm cell, an Alwa listing or an Alwa price uses the crop (`crop_in_use`): switch it off instead", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_delete_crop(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(code), _): WithRejection<Path<String>, WebError>,
) -> Result<StatusCode, WebError> {
    // No crop can have a code that is not well formed, so it is already
    // gone, which is what was asked.
    let Ok(code) = CropCode::new(code) else {
        return Ok(StatusCode::NO_CONTENT);
    };

    state
        .features
        .crop
        .delete_crop_use_case
        .execute(&staff_context, code)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
