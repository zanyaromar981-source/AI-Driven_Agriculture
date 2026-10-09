use axum::extract::State;

use super::{dtos::VersionsResponse, errors::WebError};

use crate::{
    features::versions::app::use_cases::Audience,
    infra::http::{ApiResponse, ErrorBody},
    shared::AppState,
};

/// Versions of the public kinds of data
#[utoipa::path(
    get,
    path = "/v1/versions",
    tag = "versions",
    responses(
        (status = 200, description = "Versions retrieved successfully", body = VersionsResponse),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_versions(
    State(state): State<AppState>,
) -> Result<ApiResponse<VersionsResponse>, WebError> {
    let versions = state
        .features
        .version
        .list_versions_use_case
        .execute(Audience::Public)
        .await?;

    Ok(ApiResponse::ok(VersionsResponse::from(versions.as_slice())))
}

/// Versions of every kind of data, for signed-in staff
#[utoipa::path(
    get,
    path = "/v1/dashboard/versions",
    tag = "versions",
    responses(
        (status = 200, description = "Versions retrieved successfully", body = VersionsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard_versions(
    State(state): State<AppState>,
) -> Result<ApiResponse<VersionsResponse>, WebError> {
    let versions = state
        .features
        .version
        .list_versions_use_case
        .execute(Audience::Staff)
        .await?;

    Ok(ApiResponse::ok(VersionsResponse::from(versions.as_slice())))
}
