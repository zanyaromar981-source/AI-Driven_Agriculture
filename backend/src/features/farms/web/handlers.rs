use axum::{
    Extension,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        CreateFarmParams, DashboardCreateFarmParams, DashboardFarmSummaryResponse,
        DashboardFarmsQuery, DashboardFarmsResponse, DashboardOneFarmResponse,
        DashboardRenameFarmParams, DashboardSavedFarmResponse, FarmStatsQuery, FarmStatsResponse,
        FarmStatusResponse, FarmSummaryResponse, FarmsResponse, OneFarmResponse,
        PublicFarmStatsResponse, RepaintFarmCellsParams, SavedFarmResponse,
    },
    errors::WebError,
};

use crate::{
    app::{AuthContext, Pagination, StaffContext},
    infra::http::{ApiResponse, ErrorBody, PaginationQueryDto, ValidatedJson},
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

/// Edit a farm: replace its outline, its cells and its name
///
/// The body is the one `POST /v1/farms` takes and is checked the same way.
/// The farm keeps its id. `Idempotency-Key` is accepted and not used: the
/// request carries the whole farm, so sending it again leaves the same farm
/// and answers `200` with it, with no key needed to tell a repeat apart.
#[utoipa::path(
    put,
    path = "/v1/farms/{id}",
    tag = "farms",
    params(
        ("id" = String, Path, description = "Farm ID"),
        ("Idempotency-Key" = Option<String>, Header, description = "Accepted and not used: repeating an edit is safe without it")
    ),
    request_body = CreateFarmParams,
    responses(
        (status = 200, description = "Farm edited successfully", body = SavedFarmResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn edit_farm(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<CreateFarmParams>,
) -> Result<ApiResponse<SavedFarmResponse>, WebError> {
    let input = params.into_edit_input()?;

    let (farm, dropped_cells) = state
        .features
        .farm
        .edit_farm_use_case
        .execute(&auth_context, farm_id(&id)?, input)
        .await?;

    Ok(ApiResponse::ok(SavedFarmResponse::try_from((
        &farm,
        dropped_cells.as_slice(),
    ))?))
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

/// List every farmer's farms, filtered and sorted
///
/// Newest first unless `sort` and `order` say otherwise. Every filter that
/// is given must hold.
#[utoipa::path(
    get,
    path = "/v1/dashboard/farms",
    tag = "farms",
    params(DashboardFarmsQuery, PaginationQueryDto),
    responses(
        (status = 200, description = "List of farms retrieved successfully", body = DashboardFarmsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs farms:read", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_farms(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<DashboardFarmsQuery>, WebError>,
    WithRejection(Query(page), _): WithRejection<Query<PaginationQueryDto>, WebError>,
) -> Result<ApiResponse<DashboardFarmsResponse>, WebError> {
    let pagination = Pagination::from(&page);
    let input = query.into_input(pagination)?;

    let (farms, count) = state
        .features
        .farm
        .list_all_farms_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(DashboardFarmsResponse {
        farms: farms
            .iter()
            .map(DashboardFarmSummaryResponse::from)
            .collect(),
        count,
        page: *pagination.page(),
        rows_per_page: *pagination.rows_per_page(),
    }))
}

/// Register a farm on behalf of a farmer
#[utoipa::path(
    post,
    path = "/v1/dashboard/farms",
    tag = "farms",
    request_body = DashboardCreateFarmParams,
    params(("Idempotency-Key" = Option<String>, Header, description = "Repeat it on a retry to get the farm already created")),
    responses(
        (status = 201, description = "Farm created successfully", body = DashboardSavedFarmResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs farms:create", body = ErrorBody),
        (status = 404, description = "No farmer has that phone", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_create_farm(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    headers: HeaderMap,
    ValidatedJson(params): ValidatedJson<DashboardCreateFarmParams>,
) -> Result<ApiResponse<DashboardSavedFarmResponse>, WebError> {
    let idempotency_key = headers
        .get(IDEMPOTENCY_KEY)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    let (owner, input) = params.into_input(idempotency_key)?;

    let (farm, dropped_cells) = state
        .features
        .farm
        .register_farm_for_farmer_use_case
        .execute(&staff_context, owner, input)
        .await?;

    Ok(ApiResponse::created(DashboardSavedFarmResponse::try_from(
        (&farm, dropped_cells.as_slice()),
    )?))
}

/// Get one farm with its outline, cells and owner
#[utoipa::path(
    get,
    path = "/v1/dashboard/farms/{id}",
    tag = "farms",
    params(("id" = String, Path, description = "Farm ID")),
    responses(
        (status = 200, description = "Farm retrieved successfully", body = DashboardOneFarmResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs farms:read", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_farm(
    State(state): State<AppState>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<ApiResponse<DashboardOneFarmResponse>, WebError> {
    let farm = state
        .features
        .farm
        .view_any_farm_use_case
        .execute(farm_id(&id)?)
        .await?;

    Ok(ApiResponse::ok(DashboardOneFarmResponse::try_from(&farm)?))
}

/// Rename a farm
#[utoipa::path(
    put,
    path = "/v1/dashboard/farms/{id}",
    tag = "farms",
    params(("id" = String, Path, description = "Farm ID")),
    request_body = DashboardRenameFarmParams,
    responses(
        (status = 200, description = "Farm renamed successfully", body = DashboardOneFarmResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs farms:update", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_rename_farm(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<DashboardRenameFarmParams>,
) -> Result<ApiResponse<DashboardOneFarmResponse>, WebError> {
    let input = params.into_input()?;

    let farm = state
        .features
        .farm
        .rename_farm_use_case
        .execute(&staff_context, farm_id(&id)?, input)
        .await?;

    Ok(ApiResponse::ok(DashboardOneFarmResponse::try_from(&farm)?))
}

/// Delete any farmer's farm
#[utoipa::path(
    delete,
    path = "/v1/dashboard/farms/{id}",
    tag = "farms",
    params(("id" = String, Path, description = "Farm ID")),
    responses(
        (status = 204, description = "Delete was successful, also when the farm was already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs farms:delete", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_delete_farm(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<StatusCode, WebError> {
    state
        .features
        .farm
        .remove_any_farm_use_case
        .execute(&staff_context, farm_id(&id)?)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Farms, farmers and land added up, for reports and charts
///
/// The adding is done by the database, so the dashboard never downloads
/// every farm. A farmer is counted once in the totals and once in each area
/// where they have a farm. Farms with no place are in the totals and in one
/// extra row with the slug `unknown` in each list of areas.
#[utoipa::path(
    get,
    path = "/v1/dashboard/stats/farms",
    tag = "farms",
    params(FarmStatsQuery),
    responses(
        (status = 200, description = "Totals retrieved successfully", body = FarmStatsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs farms:read", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_farm_stats(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<FarmStatsQuery>, WebError>,
) -> Result<ApiResponse<FarmStatsResponse>, WebError> {
    let input = query.into_input()?;

    let stats = state
        .features
        .farm
        .view_farm_stats_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(FarmStatsResponse::from(&stats)))
}

/// Farm totals anyone may read
///
/// Counts and areas for the whole region, by governorate, by district and
/// by crop. No farm, no name, no phone, and nothing per sub-district.
/// Answers `404` when the setting `STATS__PUBLIC_FARM_TOTALS` is `false`.
#[utoipa::path(
    get,
    path = "/v1/stats/farms",
    tag = "farms",
    responses(
        (status = 200, description = "Totals retrieved successfully", body = PublicFarmStatsResponse),
        (status = 404, description = "Public farm totals are switched off", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_public_farm_stats(
    State(state): State<AppState>,
) -> Result<ApiResponse<PublicFarmStatsResponse>, WebError> {
    let stats = state
        .features
        .farm
        .view_public_farm_stats_use_case
        .execute()
        .await?;

    Ok(ApiResponse::ok(PublicFarmStatsResponse::from(&stats)))
}
