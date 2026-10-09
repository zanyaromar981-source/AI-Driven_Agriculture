use axum::{Extension, extract::State};

use super::{
    dtos::{
        AppConfigResponse, AppVersionUsageResponse, AppVersionsResponse,
        DashboardAppConfigResponse, UpdateAppConfigParams,
    },
    errors::WebError,
};

use crate::{
    app::StaffContext,
    infra::http::{ApiResponse, ErrorBody, ValidatedJson},
    shared::AppState,
};

/// What the farmer app reads at start
///
/// Versions, maintenance, the announcement, feature switches, limits and
/// the help phone. Needs no login and is never refused for an old
/// `X-App-Version`, so an app that must update can still find that out.
#[utoipa::path(
    get,
    path = "/v1/app/config",
    tag = "app",
    responses(
        (status = 200, description = "Config retrieved successfully", body = AppConfigResponse),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_app_config(
    State(state): State<AppState>,
) -> Result<ApiResponse<AppConfigResponse>, WebError> {
    let config = state
        .features
        .app_config
        .view_app_config_use_case
        .execute()
        .await?;

    Ok(ApiResponse::ok(AppConfigResponse::from(&config)))
}

/// Get the app config, with who saved it last
#[utoipa::path(
    get,
    path = "/v1/dashboard/app/config",
    tag = "app",
    responses(
        (status = 200, description = "Config retrieved successfully", body = DashboardAppConfigResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs app:read", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_app_config(
    State(state): State<AppState>,
) -> Result<ApiResponse<DashboardAppConfigResponse>, WebError> {
    let config = state
        .features
        .app_config
        .view_app_config_use_case
        .execute()
        .await?;

    Ok(ApiResponse::ok(DashboardAppConfigResponse::from(&config)))
}

/// Replace the whole app config
///
/// Every field is replaced. A `min_version` above the version a farmer's app
/// declares refuses that app with `426` from the next request on.
#[utoipa::path(
    put,
    path = "/v1/dashboard/app/config",
    tag = "app",
    request_body = UpdateAppConfigParams,
    responses(
        (status = 200, description = "Config replaced successfully", body = DashboardAppConfigResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs app:update", body = ErrorBody),
        (status = 422, description = "`bad_version`, `min_above_latest`, `bad_limits`, `bad_window`, `missing_text` or `invalid`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_update_app_config(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    ValidatedJson(params): ValidatedJson<UpdateAppConfigParams>,
) -> Result<ApiResponse<DashboardAppConfigResponse>, WebError> {
    let settings = params.into_input()?;

    let config = state
        .features
        .app_config
        .update_app_config_use_case
        .execute(&staff_context, settings)
        .await?;

    Ok(ApiResponse::ok(DashboardAppConfigResponse::from(&config)))
}

/// Which versions of the app farmers used in the last 30 days
///
/// Newest version first. A farmer is counted once, under the version their
/// most recent request came from.
#[utoipa::path(
    get,
    path = "/v1/dashboard/app/versions",
    tag = "app",
    responses(
        (status = 200, description = "Versions retrieved successfully", body = AppVersionsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs app:read", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_app_versions(
    State(state): State<AppState>,
) -> Result<ApiResponse<AppVersionsResponse>, WebError> {
    let usage = state
        .features
        .app_config
        .list_app_versions_use_case
        .execute()
        .await?;

    Ok(ApiResponse::ok(AppVersionsResponse {
        versions: usage.iter().map(AppVersionUsageResponse::from).collect(),
    }))
}
