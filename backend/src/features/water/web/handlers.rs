use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        SavedWaterPlanEntryResponse, SetWaterPlanEntryParams, WaterPlanQuery, WaterPlanResponse,
    },
    errors::WebError,
};

use crate::{
    features::water::{
        app::AppError,
        domain::{Season, ZoneSlug},
    },
    infra::http::{ApiResponse, ErrorBody, ValidatedJson},
    shared::AppState,
};

/// Get the plan for which zones should receive water first
#[utoipa::path(
    get,
    path = "/v1/water/plan",
    tag = "water",
    params(WaterPlanQuery),
    responses(
        (status = 200, description = "Water plan retrieved successfully", body = WaterPlanResponse),
        (status = 404, description = "No water plan made yet", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_water_plan(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<WaterPlanQuery>, WebError>,
) -> Result<ApiResponse<WaterPlanResponse>, WebError> {
    let plan = state
        .features
        .water
        .view_water_plan_use_case
        .execute(query.into_input()?)
        .await?;

    Ok(ApiResponse::ok(WaterPlanResponse::from(&plan)))
}

/// Store one zone's entry in a season's water plan, replacing an earlier one
#[utoipa::path(
    put,
    path = "/v1/ingest/water/plan/{season}/zones/{zone_slug}",
    tag = "water",
    params(
        ("season" = String, Path, description = "Season, for example 2026-27"),
        ("zone_slug" = String, Path, description = "Zone slug, for example chamchamal"),
        ("X-Service-Key" = String, Header, description = "The data jobs' shared key")
    ),
    request_body = SetWaterPlanEntryParams,
    responses(
        (status = 200, description = "Entry stored", body = SavedWaterPlanEntryResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_water_plan_entry(
    State(state): State<AppState>,
    WithRejection(Path((season, zone_slug)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<SetWaterPlanEntryParams>,
) -> Result<ApiResponse<SavedWaterPlanEntryResponse>, WebError> {
    let input = params.into_input(season, zone_slug)?;

    let entry = state
        .features
        .water
        .set_water_plan_entry_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(SavedWaterPlanEntryResponse::from(&entry)))
}

/// Remove one zone's entry from a season's water plan
#[utoipa::path(
    delete,
    path = "/v1/ingest/water/plan/{season}/zones/{zone_slug}",
    tag = "water",
    params(
        ("season" = String, Path, description = "Season, for example 2026-27"),
        ("zone_slug" = String, Path, description = "Zone slug, for example chamchamal"),
        ("X-Service-Key" = String, Header, description = "The data jobs' shared key")
    ),
    responses(
        (status = 204, description = "Delete was successful"),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 404, description = "Entry not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn delete_water_plan_entry(
    State(state): State<AppState>,
    WithRejection(Path((season, zone_slug)), _): WithRejection<Path<(String, String)>, WebError>,
) -> Result<StatusCode, WebError> {
    let season = Season::new(season).map_err(AppError::from)?;
    let zone_slug = ZoneSlug::new(zone_slug).map_err(AppError::from)?;

    state
        .features
        .water
        .remove_water_plan_entry_use_case
        .execute(season, zone_slug)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
