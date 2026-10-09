use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;
use chrono::Utc;

use super::{
    dtos::{
        FarmHistoryResponse, HistoryCoverageResponse, HistoryFarmCoverageResponse, HistoryMetric,
        HistoryQueryParams, HistoryRecordedResponse, RecordFarmHistoryParams,
    },
    errors::WebError,
};

use crate::{
    app::{AuthContext, StaffContext},
    infra::http::{ApiResponse, ErrorBody, ValidatedJson},
    shared::AppState,
};

/// Farm ids travel as opaque strings. One that is not a number cannot name
/// a farm, so it is not found rather than a bad request.
fn farm_id(raw: &str) -> Result<i32, WebError> {
    raw.parse().map_err(|_| WebError::not_found())
}

/// Get up to ten years of monthly history of one of the farmer's own farms
#[utoipa::path(
    get,
    path = "/v1/farms/{id}/history",
    tag = "history",
    params(("id" = String, Path, description = "Farm ID"), HistoryQueryParams),
    responses(
        (status = 200, description = "History retrieved successfully", body = FarmHistoryResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 422, description = "Unknown metric, a month that is not YYYY-MM, or a window of more than 120 months (`bad_window`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_farm_history(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
    WithRejection(Query(params), _): WithRejection<Query<HistoryQueryParams>, WebError>,
) -> Result<ApiResponse<FarmHistoryResponse>, WebError> {
    let farm_id = farm_id(&id)?;

    let series = state
        .features
        .history
        .view_farm_history_use_case
        .execute(&auth_context, farm_id, params.into_query()?, Utc::now())
        .await?;

    Ok(ApiResponse::ok(FarmHistoryResponse::from((
        farm_id,
        series.as_slice(),
    ))))
}

/// List every farm with how much history is stored for it, for the data job
#[utoipa::path(
    get,
    path = "/v1/ingest/farms/history/coverage",
    tag = "history",
    params(("X-Service-Key" = String, Header, description = "The ingest service key")),
    responses(
        (status = 200, description = "Coverage retrieved successfully", body = HistoryCoverageResponse),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_history_coverage(
    State(state): State<AppState>,
) -> Result<ApiResponse<HistoryCoverageResponse>, WebError> {
    let coverage = state
        .features
        .history
        .list_history_coverage_use_case
        .execute()
        .await?;

    Ok(ApiResponse::ok(HistoryCoverageResponse {
        farms: coverage
            .iter()
            .map(HistoryFarmCoverageResponse::from)
            .collect(),
    }))
}

/// Store months of one metric of a farm, keeping the months not named
#[utoipa::path(
    put,
    path = "/v1/ingest/farms/{id}/history/{metric}",
    tag = "history",
    params(
        ("id" = String, Path, description = "Farm ID"),
        ("metric" = HistoryMetric, Path, description = "The metric the months belong to"),
        ("X-Service-Key" = String, Header, description = "The ingest service key")
    ),
    request_body = RecordFarmHistoryParams,
    responses(
        (status = 200, description = "Months stored", body = HistoryRecordedResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 422, description = "Unknown metric, or a rule is broken: `wrong_unit`, `bad_points`, `duplicate_month`, `value_out_of_range`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_farm_history(
    State(state): State<AppState>,
    WithRejection(Path((id, metric)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<RecordFarmHistoryParams>,
) -> Result<ApiResponse<HistoryRecordedResponse>, WebError> {
    let input = params.into_input(farm_id(&id)?, HistoryMetric::from_path(&metric)?)?;

    let stored = state
        .features
        .history
        .record_farm_history_use_case
        .execute(input, Utc::now())
        .await?;

    Ok(ApiResponse::ok(HistoryRecordedResponse::from(&stored)))
}

/// Get the stored monthly history of any farmer's farm
#[utoipa::path(
    get,
    path = "/v1/dashboard/farms/{id}/history",
    tag = "history",
    params(("id" = String, Path, description = "Farm ID"), HistoryQueryParams),
    responses(
        (status = 200, description = "History retrieved successfully", body = FarmHistoryResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs insights:read", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 422, description = "Unknown metric, a month that is not YYYY-MM, or a window of more than 120 months (`bad_window`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard_farm_history(
    State(state): State<AppState>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
    WithRejection(Query(params), _): WithRejection<Query<HistoryQueryParams>, WebError>,
) -> Result<ApiResponse<FarmHistoryResponse>, WebError> {
    let farm_id = farm_id(&id)?;

    let series = state
        .features
        .history
        .view_stored_farm_history_use_case
        .execute(farm_id, params.into_query()?, Utc::now())
        .await?;

    Ok(ApiResponse::ok(FarmHistoryResponse::from((
        farm_id,
        series.as_slice(),
    ))))
}

/// Clear one metric's whole series of a farm, so the data job fetches it again
#[utoipa::path(
    delete,
    path = "/v1/dashboard/farms/{id}/history/{metric}",
    tag = "history",
    params(
        ("id" = String, Path, description = "Farm ID"),
        ("metric" = HistoryMetric, Path, description = "The metric to clear")
    ),
    responses(
        (status = 204, description = "Delete was successful, also when the series was already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs insights:delete", body = ErrorBody),
        (status = 404, description = "The farm id is not a number", body = ErrorBody),
        (status = 422, description = "Unknown metric", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_dashboard_farm_history(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((id, metric)), _): WithRejection<Path<(String, String)>, WebError>,
) -> Result<StatusCode, WebError> {
    state
        .features
        .history
        .clear_farm_history_use_case
        .execute(
            &staff_context,
            farm_id(&id)?,
            HistoryMetric::from_path(&metric)?,
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
