use axum::{
    Extension,
    extract::{Path, State},
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        FarmCoverageResponse, FarmInsightsResponse, FarmsCoverageResponse, RecordFarmInsightParams,
        Topic, TopicInsightResponse,
    },
    errors::WebError,
};

use crate::{
    app::AuthContext,
    infra::http::{ApiResponse, ErrorBody, ValidatedJson},
    shared::AppState,
};

/// Farm ids travel as opaque strings. One that is not a number cannot name
/// a farm, so it is not found rather than a bad request.
fn farm_id(raw: &str) -> Result<i32, WebError> {
    raw.parse().map_err(|_| WebError::not_found())
}

/// Get the current readings of one of the farmer's own farms
#[utoipa::path(
    get,
    path = "/v1/farms/{id}/insights",
    tag = "insights",
    params(("id" = String, Path, description = "Farm ID")),
    responses(
        (status = 200, description = "Insights retrieved successfully", body = FarmInsightsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_farm_insights(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<ApiResponse<FarmInsightsResponse>, WebError> {
    let farm_id = farm_id(&id)?;

    let insights = state
        .features
        .insight
        .view_farm_insights_use_case
        .execute(&auth_context, farm_id)
        .await?;

    Ok(ApiResponse::ok(FarmInsightsResponse::from((
        farm_id,
        insights.as_slice(),
    ))))
}

/// List every farm with the readings it already has, for the data jobs
#[utoipa::path(
    get,
    path = "/v1/ingest/farms",
    tag = "insights",
    params(("X-Service-Key" = String, Header, description = "The ingest service key")),
    responses(
        (status = 200, description = "Farms retrieved successfully", body = FarmsCoverageResponse),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_farm_coverage(
    State(state): State<AppState>,
) -> Result<ApiResponse<FarmsCoverageResponse>, WebError> {
    let coverage = state
        .features
        .insight
        .list_farm_coverage_use_case
        .execute()
        .await?;

    Ok(ApiResponse::ok(FarmsCoverageResponse {
        farms: coverage.iter().map(FarmCoverageResponse::from).collect(),
    }))
}

/// Store a farm's reading for one topic, replacing the one it had
#[utoipa::path(
    put,
    path = "/v1/ingest/farms/{id}/insights/{topic}",
    tag = "insights",
    params(
        ("id" = String, Path, description = "Farm ID"),
        ("topic" = Topic, Path, description = "The topic the reading is about"),
        ("X-Service-Key" = String, Header, description = "The ingest service key")
    ),
    request_body = RecordFarmInsightParams,
    responses(
        (status = 200, description = "Reading stored", body = TopicInsightResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 404, description = "The farm id is not a number", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_farm_insight(
    State(state): State<AppState>,
    WithRejection(Path((id, topic)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<RecordFarmInsightParams>,
) -> Result<ApiResponse<TopicInsightResponse>, WebError> {
    let input = params.into_input(farm_id(&id)?, Topic::from_path(&topic)?)?;

    let insight = state
        .features
        .insight
        .record_farm_insight_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(TopicInsightResponse::from(&insight)))
}
