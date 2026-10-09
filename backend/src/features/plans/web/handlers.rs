use axum::{
    Extension,
    extract::{Path, State},
};
use axum_extra::extract::WithRejection;
use chrono::Utc;

use super::{
    dtos::{
        FarmPlanResponse, PlanCoverageFarmResponse, PlanCoverageResponse, PlanDashboardResponse,
        PlanDashboardStoredResponse, RecordFarmPlanParams,
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

/// Get this week's plan of one of the farmer's own farms
#[utoipa::path(
    get,
    path = "/v1/farms/{id}/plan",
    tag = "plans",
    params(("id" = String, Path, description = "Farm ID")),
    responses(
        (status = 200, description = "The plan, starting today at the earliest", body = FarmPlanResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Farm not found (`not_found`), no plan yet (`plan_not_ready`) or the stored plan is more than 2 days old (`plan_stale`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_farm_plan(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<ApiResponse<FarmPlanResponse>, WebError> {
    let plan = state
        .features
        .plan
        .view_farm_plan_use_case
        .execute(&auth_context, farm_id(&id)?, Utc::now())
        .await?;

    Ok(ApiResponse::ok(FarmPlanResponse::from(&plan)))
}

/// List every farm with the time its plan was issued, for the plan job
#[utoipa::path(
    get,
    path = "/v1/ingest/farms/plans/coverage",
    tag = "plans",
    params(("X-Service-Key" = String, Header, description = "The ingest service key")),
    responses(
        (status = 200, description = "Farms retrieved successfully", body = PlanCoverageResponse),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_plan_coverage(
    State(state): State<AppState>,
) -> Result<ApiResponse<PlanCoverageResponse>, WebError> {
    let coverage = state
        .features
        .plan
        .list_plan_coverage_use_case
        .execute()
        .await?;

    Ok(ApiResponse::ok(PlanCoverageResponse {
        farms: coverage
            .iter()
            .map(PlanCoverageFarmResponse::from)
            .collect(),
    }))
}

/// Store a farm's plan, replacing the one it had unless that one is newer
#[utoipa::path(
    put,
    path = "/v1/ingest/farms/{id}/plan",
    tag = "plans",
    params(
        ("id" = String, Path, description = "Farm ID"),
        ("X-Service-Key" = String, Header, description = "The ingest service key")
    ),
    request_body = RecordFarmPlanParams,
    responses(
        (status = 200, description = "The plan that is stored now: the pushed one, or the one already there when it was issued later", body = FarmPlanResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 422, description = "Validation error (`too_many_days`, `unequal_days`, `alert_outside_plan`, `too_many_alerts`, `duplicate_decision`, `issued_in_future`, `bad_from`, `invalid`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_farm_plan(
    State(state): State<AppState>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<RecordFarmPlanParams>,
) -> Result<ApiResponse<FarmPlanResponse>, WebError> {
    let input = params.into_input(farm_id(&id)?)?;

    let plan = state
        .features
        .plan
        .record_farm_plan_use_case
        .execute(input, Utc::now())
        .await?;

    Ok(ApiResponse::ok(FarmPlanResponse::from(&plan)))
}

/// Show the stored plan of any farmer's farm, as the job pushed it
#[utoipa::path(
    get,
    path = "/v1/dashboard/farms/{id}/plan",
    tag = "plans",
    params(("id" = String, Path, description = "Farm ID")),
    responses(
        (status = 200, description = "The stored plan, or `plan: null` when the farm has none yet", body = PlanDashboardResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs insights:read", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard_farm_plan(
    State(state): State<AppState>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<ApiResponse<PlanDashboardResponse>, WebError> {
    let farm_id = farm_id(&id)?;
    let now = Utc::now();

    let plan = state
        .features
        .plan
        .view_stored_farm_plan_use_case
        .execute(farm_id)
        .await?;

    Ok(ApiResponse::ok(PlanDashboardResponse {
        farm_id: farm_id.to_string(),
        plan: plan.map(|plan| PlanDashboardStoredResponse {
            plan: FarmPlanResponse::from(&plan),
            stale: plan.is_stale(now),
            updated_at: *plan.updated_at(),
        }),
    }))
}
