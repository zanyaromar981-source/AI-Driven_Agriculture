use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        CreateWaterPlanEntryDashboardParams, SavedWaterPlanEntryResponse, SetWaterPlanEntryParams,
        WaterPlanEntriesResponse, WaterPlanQuery, WaterPlanResponse, WaterSeasonsResponse,
    },
    errors::WebError,
};

use crate::{
    app::StaffContext,
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

/// List the seasons that have a water plan entry, newest first
#[utoipa::path(
    get,
    path = "/v1/dashboard/water/seasons",
    tag = "water",
    responses(
        (status = 200, description = "Seasons retrieved successfully", body = WaterSeasonsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs water:read", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_water_seasons(
    State(state): State<AppState>,
) -> Result<ApiResponse<WaterSeasonsResponse>, WebError> {
    let seasons = state
        .features
        .water
        .list_water_seasons_use_case
        .execute()
        .await?;

    Ok(ApiResponse::ok(WaterSeasonsResponse {
        seasons: seasons.iter().map(Into::into).collect(),
    }))
}

/// List one season's stored water plan entries, highest need first
#[utoipa::path(
    get,
    path = "/v1/dashboard/water/plan/{season}/entries",
    tag = "water",
    params(("season" = String, Path, description = "Season, for example 2026-27")),
    responses(
        (status = 200, description = "Entries retrieved successfully", body = WaterPlanEntriesResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs water:read", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_water_plan_entries(
    State(state): State<AppState>,
    WithRejection(Path(season), _): WithRejection<Path<String>, WebError>,
) -> Result<ApiResponse<WaterPlanEntriesResponse>, WebError> {
    let season = Season::new(season).map_err(AppError::from)?;

    let entries = state
        .features
        .water
        .list_water_plan_entries_use_case
        .execute(season)
        .await?;

    Ok(ApiResponse::ok(WaterPlanEntriesResponse {
        entries: entries
            .iter()
            .map(|entry| SavedWaterPlanEntryResponse::from(entry).entry)
            .collect(),
    }))
}

/// Add a zone to a season's water plan
#[utoipa::path(
    post,
    path = "/v1/dashboard/water/plan/{season}/entries",
    tag = "water",
    params(("season" = String, Path, description = "Season, for example 2026-27")),
    request_body = CreateWaterPlanEntryDashboardParams,
    responses(
        (status = 201, description = "Entry created", body = SavedWaterPlanEntryResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs water:create", body = ErrorBody),
        (status = 409, description = "The season's plan already has that zone (`already_exists`)", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_create_water_plan_entry(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(season), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<CreateWaterPlanEntryDashboardParams>,
) -> Result<ApiResponse<SavedWaterPlanEntryResponse>, WebError> {
    let input = params.into_input(season)?;

    let entry = state
        .features
        .water
        .create_water_plan_entry_use_case
        .execute(&staff_context, input)
        .await?;

    Ok(ApiResponse::created(SavedWaterPlanEntryResponse::from(
        &entry,
    )))
}

/// Replace one zone's entry in a season's water plan
#[utoipa::path(
    put,
    path = "/v1/dashboard/water/plan/{season}/entries/{zone_slug}",
    tag = "water",
    params(
        ("season" = String, Path, description = "Season, for example 2026-27"),
        ("zone_slug" = String, Path, description = "Zone slug, for example chamchamal")
    ),
    request_body = SetWaterPlanEntryParams,
    responses(
        (status = 200, description = "Entry updated", body = SavedWaterPlanEntryResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs water:update", body = ErrorBody),
        (status = 404, description = "The season's plan has no entry for that zone", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_update_water_plan_entry(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((season, zone_slug)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<SetWaterPlanEntryParams>,
) -> Result<ApiResponse<SavedWaterPlanEntryResponse>, WebError> {
    let input = params.into_input(season, zone_slug)?;

    let entry = state
        .features
        .water
        .update_water_plan_entry_use_case
        .execute(&staff_context, input)
        .await?;

    Ok(ApiResponse::ok(SavedWaterPlanEntryResponse::from(&entry)))
}

/// Remove one zone's entry from a season's water plan
#[utoipa::path(
    delete,
    path = "/v1/dashboard/water/plan/{season}/entries/{zone_slug}",
    tag = "water",
    params(
        ("season" = String, Path, description = "Season, for example 2026-27"),
        ("zone_slug" = String, Path, description = "Zone slug, for example chamchamal")
    ),
    responses(
        (status = 204, description = "Delete was successful, or the entry was already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs water:delete", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_delete_water_plan_entry(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((season, zone_slug)), _): WithRejection<Path<(String, String)>, WebError>,
) -> Result<StatusCode, WebError> {
    let season = Season::new(season).map_err(AppError::from)?;
    let zone_slug = ZoneSlug::new(zone_slug).map_err(AppError::from)?;

    state
        .features
        .water
        .delete_water_plan_entry_use_case
        .execute(&staff_context, season, zone_slug)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
