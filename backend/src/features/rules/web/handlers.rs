use axum::{
    Extension,
    extract::{Path, Query, State},
};
use axum_extra::extract::WithRejection;
use chrono::Utc;

use super::{
    dtos::{
        ChangeRuleParams, OneRuleResponse, ResetRuleParams, RuleChangeResponse,
        RuleHistoryResponse, RuleResponse, RuleValueResponse, RuleValuesQuery, RuleValuesResponse,
        RulesResponse,
    },
    errors::WebError,
};

use crate::{
    app::{Pagination, StaffContext},
    features::rules::domain::RuleCode,
    infra::http::{ApiResponse, ErrorBody, PaginationQueryDto, ValidatedJson},
    shared::AppState,
};

/// A code that is not well formed cannot name a rule, so it is not found
/// rather than a bad request.
fn rule_code(raw: String) -> Result<RuleCode, WebError> {
    RuleCode::new(raw).map_err(|_| WebError::not_found())
}

/// The values the data jobs read once per run
#[utoipa::path(
    get,
    path = "/v1/ingest/rules",
    tag = "rules",
    params(
        RuleValuesQuery,
        ("X-Service-Key" = String, Header, description = "The data jobs' shared key")
    ),
    responses(
        (status = 200, description = "Rule values retrieved successfully", body = RuleValuesResponse),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 422, description = "Unknown `used_by`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_rule_values(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<RuleValuesQuery>, WebError>,
) -> Result<ApiResponse<RuleValuesResponse>, WebError> {
    let used_by = query.into_input()?;

    let rules = state
        .features
        .rule
        .list_rules_use_case
        .execute(used_by)
        .await?;

    Ok(ApiResponse::ok(RuleValuesResponse {
        rules: rules.iter().map(RuleValueResponse::from).collect(),
    }))
}

/// List every rule as stored, for the dashboard's editing screen
#[utoipa::path(
    get,
    path = "/v1/dashboard/rules",
    tag = "rules",
    responses(
        (status = 200, description = "Rules retrieved successfully", body = RulesResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs rules:read", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_rules(
    State(state): State<AppState>,
) -> Result<ApiResponse<RulesResponse>, WebError> {
    let rules = state
        .features
        .rule
        .list_rules_use_case
        .execute(None)
        .await?;

    Ok(ApiResponse::ok(RulesResponse {
        rules: rules.iter().map(RuleResponse::from).collect(),
    }))
}

/// Change one rule's value, with the reason, and log the change
#[utoipa::path(
    put,
    path = "/v1/dashboard/rules/{code}",
    tag = "rules",
    params(("code" = String, Path, description = "Rule code, for example frost_c")),
    request_body = ChangeRuleParams,
    responses(
        (status = 200, description = "Rule changed, or it already held the value", body = OneRuleResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs rules:update", body = ErrorBody),
        (status = 404, description = "Rule not found", body = ErrorBody),
        (status = 422, description = "Value outside the rule's range (`bad_range`), or a reason not 3 to 500 characters long (`bad_reason`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_change_rule(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(code), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<ChangeRuleParams>,
) -> Result<ApiResponse<OneRuleResponse>, WebError> {
    let input = params.into_input(rule_code(code)?)?;

    let outcome = state
        .features
        .rule
        .change_rule_use_case
        .execute(&staff_context, input, Utc::now())
        .await?;

    Ok(ApiResponse::ok(OneRuleResponse::from(&outcome)))
}

/// List one rule's changes, newest first
#[utoipa::path(
    get,
    path = "/v1/dashboard/rules/{code}/history",
    tag = "rules",
    params(
        ("code" = String, Path, description = "Rule code, for example frost_c"),
        PaginationQueryDto
    ),
    responses(
        (status = 200, description = "History retrieved successfully", body = RuleHistoryResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs rules:read", body = ErrorBody),
        (status = 404, description = "Rule not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_rule_history(
    State(state): State<AppState>,
    WithRejection(Path(code), _): WithRejection<Path<String>, WebError>,
    WithRejection(Query(page), _): WithRejection<Query<PaginationQueryDto>, WebError>,
) -> Result<ApiResponse<RuleHistoryResponse>, WebError> {
    let code = rule_code(code)?;
    let pagination = Pagination::from(&page);

    let (changes, count) = state
        .features
        .rule
        .view_rule_history_use_case
        .execute(&code, pagination)
        .await?;

    Ok(ApiResponse::ok(RuleHistoryResponse {
        changes: changes.iter().map(RuleChangeResponse::from).collect(),
        count,
        page: *pagination.page(),
        rows_per_page: *pagination.rows_per_page(),
    }))
}

/// Put one rule back to its default value, with the reason, and log it
#[utoipa::path(
    post,
    path = "/v1/dashboard/rules/{code}/reset",
    tag = "rules",
    params(("code" = String, Path, description = "Rule code, for example frost_c")),
    request_body = ResetRuleParams,
    responses(
        (status = 200, description = "Rule reset, or it already held its default", body = OneRuleResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs rules:update", body = ErrorBody),
        (status = 404, description = "Rule not found", body = ErrorBody),
        (status = 422, description = "A reason not 3 to 500 characters long (`bad_reason`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_reset_rule(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(code), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<ResetRuleParams>,
) -> Result<ApiResponse<OneRuleResponse>, WebError> {
    let input = params.into_input(rule_code(code)?)?;

    let outcome = state
        .features
        .rule
        .change_rule_use_case
        .execute(&staff_context, input, Utc::now())
        .await?;

    Ok(ApiResponse::ok(OneRuleResponse::from(&outcome)))
}
