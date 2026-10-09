use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        FarmCoverageResponse, FarmInsightsResponse, FarmsCoverageResponse,
        InsightDashboardCreateParams, InsightDashboardListResponse, InsightDashboardOneResponse,
        InsightDashboardResponse, RecordFarmInsightParams, Topic, TopicInsightResponse,
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

/// List every stored reading of any farmer's farm, in the fixed topic order
#[utoipa::path(
    get,
    path = "/v1/dashboard/farms/{id}/insights",
    tag = "insights",
    params(("id" = String, Path, description = "Farm ID")),
    responses(
        (status = 200, description = "Insights retrieved successfully", body = InsightDashboardListResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs insights:read", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard_farm_insights(
    State(state): State<AppState>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<ApiResponse<InsightDashboardListResponse>, WebError> {
    let farm_id = farm_id(&id)?;

    let insights = state
        .features
        .insight
        .view_stored_farm_insights_use_case
        .execute(farm_id)
        .await?;

    Ok(ApiResponse::ok(InsightDashboardListResponse {
        farm_id: farm_id.to_string(),
        insights: insights.iter().map(Into::into).collect(),
    }))
}

/// Enter a reading by hand for a topic the farm has none for
#[utoipa::path(
    post,
    path = "/v1/dashboard/farms/{id}/insights",
    tag = "insights",
    params(("id" = String, Path, description = "Farm ID")),
    request_body = InsightDashboardCreateParams,
    responses(
        (status = 201, description = "Reading created successfully", body = InsightDashboardOneResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs insights:create", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 409, description = "The farm already has a reading for this topic (`already_exists`)", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_dashboard_farm_insight(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<InsightDashboardCreateParams>,
) -> Result<ApiResponse<InsightDashboardOneResponse>, WebError> {
    let farm_id = farm_id(&id)?;
    let input = params.into_input(farm_id)?;

    let insight = state
        .features
        .insight
        .create_farm_insight_use_case
        .execute(&staff_context, input)
        .await?;

    Ok(ApiResponse::created(InsightDashboardOneResponse {
        farm_id: farm_id.to_string(),
        insight: InsightDashboardResponse::from(&insight),
    }))
}

/// Replace the reading a farm has for one topic
#[utoipa::path(
    put,
    path = "/v1/dashboard/farms/{id}/insights/{topic}",
    tag = "insights",
    params(
        ("id" = String, Path, description = "Farm ID"),
        ("topic" = Topic, Path, description = "The topic the reading is about")
    ),
    request_body = RecordFarmInsightParams,
    responses(
        (status = 200, description = "Reading updated successfully", body = InsightDashboardOneResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs insights:update", body = ErrorBody),
        (status = 404, description = "The farm has no reading for this topic", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_dashboard_farm_insight(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((id, topic)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<RecordFarmInsightParams>,
) -> Result<ApiResponse<InsightDashboardOneResponse>, WebError> {
    let farm_id = farm_id(&id)?;
    let input = params.into_input(farm_id, Topic::from_path(&topic)?)?;

    let insight = state
        .features
        .insight
        .correct_farm_insight_use_case
        .execute(&staff_context, input)
        .await?;

    Ok(ApiResponse::ok(InsightDashboardOneResponse {
        farm_id: farm_id.to_string(),
        insight: InsightDashboardResponse::from(&insight),
    }))
}

/// Remove the reading a farm has for one topic
#[utoipa::path(
    delete,
    path = "/v1/dashboard/farms/{id}/insights/{topic}",
    tag = "insights",
    params(
        ("id" = String, Path, description = "Farm ID"),
        ("topic" = Topic, Path, description = "The topic the reading is about")
    ),
    responses(
        (status = 204, description = "Delete was successful, also when the reading was already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs insights:delete", body = ErrorBody),
        (status = 404, description = "The farm id is not a number", body = ErrorBody),
        (status = 422, description = "Unknown topic", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_dashboard_farm_insight(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path((id, topic)), _): WithRejection<Path<(String, String)>, WebError>,
) -> Result<StatusCode, WebError> {
    state
        .features
        .insight
        .remove_farm_insight_use_case
        .execute(&staff_context, farm_id(&id)?, Topic::from_path(&topic)?)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
