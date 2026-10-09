use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        DashboardCreateFarmerParams, DashboardFarmerResponse, DashboardFarmersQuery,
        DashboardFarmersResponse, DashboardOneFarmerResponse, DashboardUpdateFarmerParams,
        EditProfileParams, ProfileResponse, SendSignInCodeParams, SignInCodeSentResponse,
        SignedInResponse, VerifySignInCodeParams,
    },
    errors::WebError,
};

use crate::{
    app::{AuthContext, Pagination, StaffContext},
    infra::http::{ApiResponse, ErrorBody, PaginationQueryDto, ValidatedJson},
    shared::AppState,
};

/// Farmer ids travel as opaque strings. One that is not a number cannot
/// name a farmer, so it is not found rather than a bad request.
fn farmer_id(raw: &str) -> Result<i32, WebError> {
    raw.parse().map_err(|_| WebError::not_found())
}

/// Send a sign-in code to a phone
#[utoipa::path(
    post,
    path = "/v1/auth/otp/send",
    tag = "farmers",
    request_body = SendSignInCodeParams,
    responses(
        (status = 200, description = "Code sent", body = SignInCodeSentResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 429, description = "A code was sent a moment ago", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn send_sign_in_code(
    State(state): State<AppState>,
    ValidatedJson(params): ValidatedJson<SendSignInCodeParams>,
) -> Result<ApiResponse<SignInCodeSentResponse>, WebError> {
    let input = params.into_input()?;

    let requested = state
        .features
        .farmer
        .request_sign_in_code_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(requested.into()))
}

/// Exchange a sign-in code for a token
#[utoipa::path(
    post,
    path = "/v1/auth/otp/verify",
    tag = "farmers",
    request_body = VerifySignInCodeParams,
    responses(
        (status = 200, description = "Signed in", body = SignedInResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Wrong or expired code", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn verify_sign_in_code(
    State(state): State<AppState>,
    ValidatedJson(params): ValidatedJson<VerifySignInCodeParams>,
) -> Result<ApiResponse<SignedInResponse>, WebError> {
    let input = params.into_input()?;

    let signed_in = state
        .features
        .farmer
        .verify_sign_in_code_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(signed_in.into()))
}

/// Get the authenticated farmer's profile
#[utoipa::path(
    get,
    path = "/v1/me",
    tag = "farmers",
    responses(
        (status = 200, description = "Profile retrieved successfully", body = ProfileResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Farmer not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_profile(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
) -> Result<ApiResponse<ProfileResponse>, WebError> {
    let farmer = state
        .features
        .farmer
        .view_profile_use_case
        .execute(&auth_context)
        .await?;

    Ok(ApiResponse::ok(ProfileResponse::from(&farmer)))
}

/// Update the authenticated farmer's profile
#[utoipa::path(
    put,
    path = "/v1/me",
    tag = "farmers",
    request_body = EditProfileParams,
    responses(
        (status = 200, description = "Profile updated successfully", body = ProfileResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Farmer not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_profile(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    ValidatedJson(params): ValidatedJson<EditProfileParams>,
) -> Result<ApiResponse<ProfileResponse>, WebError> {
    let input = params.into_input()?;

    let farmer = state
        .features
        .farmer
        .edit_profile_use_case
        .execute(&auth_context, input)
        .await?;

    Ok(ApiResponse::ok(ProfileResponse::from(&farmer)))
}

/// List farmers, newest first
#[utoipa::path(
    get,
    path = "/v1/dashboard/farmers",
    tag = "farmers",
    params(DashboardFarmersQuery, PaginationQueryDto),
    responses(
        (status = 200, description = "List of farmers retrieved successfully", body = DashboardFarmersResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs farmers:read", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_farmers(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<DashboardFarmersQuery>, WebError>,
    WithRejection(Query(page), _): WithRejection<Query<PaginationQueryDto>, WebError>,
) -> Result<ApiResponse<DashboardFarmersResponse>, WebError> {
    let pagination = Pagination::from(&page);
    let input = query.into_input(pagination)?;

    let (farmers, count) = state
        .features
        .farmer
        .list_farmers_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(DashboardFarmersResponse {
        farmers: farmers
            .iter()
            .map(DashboardFarmerResponse::try_from)
            .collect::<Result<Vec<_>, _>>()?,
        count,
        page: *pagination.page(),
        rows_per_page: *pagination.rows_per_page(),
    }))
}

/// Register a farmer without a sign-in code
#[utoipa::path(
    post,
    path = "/v1/dashboard/farmers",
    tag = "farmers",
    request_body = DashboardCreateFarmerParams,
    responses(
        (status = 201, description = "Farmer created successfully", body = DashboardOneFarmerResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs farmers:create", body = ErrorBody),
        (status = 409, description = "The phone already has a farmer (`already_exists`)", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_create_farmer(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    ValidatedJson(params): ValidatedJson<DashboardCreateFarmerParams>,
) -> Result<ApiResponse<DashboardOneFarmerResponse>, WebError> {
    let input = params.into_input()?;

    let farmer = state
        .features
        .farmer
        .register_farmer_use_case
        .execute(&staff_context, input)
        .await?;

    Ok(ApiResponse::created(DashboardOneFarmerResponse::try_from(
        &farmer,
    )?))
}

/// Get one farmer
#[utoipa::path(
    get,
    path = "/v1/dashboard/farmers/{id}",
    tag = "farmers",
    params(("id" = String, Path, description = "Farmer ID")),
    responses(
        (status = 200, description = "Farmer retrieved successfully", body = DashboardOneFarmerResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs farmers:read", body = ErrorBody),
        (status = 404, description = "Farmer not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_farmer(
    State(state): State<AppState>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<ApiResponse<DashboardOneFarmerResponse>, WebError> {
    let farmer = state
        .features
        .farmer
        .view_farmer_use_case
        .execute(farmer_id(&id)?)
        .await?;

    Ok(ApiResponse::ok(DashboardOneFarmerResponse::try_from(
        &farmer,
    )?))
}

/// Change a farmer's name and language
#[utoipa::path(
    put,
    path = "/v1/dashboard/farmers/{id}",
    tag = "farmers",
    params(("id" = String, Path, description = "Farmer ID")),
    request_body = DashboardUpdateFarmerParams,
    responses(
        (status = 200, description = "Farmer updated successfully", body = DashboardOneFarmerResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs farmers:update", body = ErrorBody),
        (status = 404, description = "Farmer not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_update_farmer(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<DashboardUpdateFarmerParams>,
) -> Result<ApiResponse<DashboardOneFarmerResponse>, WebError> {
    let input = params.into_input()?;

    let farmer = state
        .features
        .farmer
        .edit_farmer_use_case
        .execute(&staff_context, farmer_id(&id)?, input)
        .await?;

    Ok(ApiResponse::ok(DashboardOneFarmerResponse::try_from(
        &farmer,
    )?))
}

/// Delete a farmer with their open sign-in code and all their farms
#[utoipa::path(
    delete,
    path = "/v1/dashboard/farmers/{id}",
    tag = "farmers",
    params(("id" = String, Path, description = "Farmer ID")),
    responses(
        (status = 204, description = "Delete was successful, also when the farmer was already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs farmers:delete", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_delete_farmer(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<StatusCode, WebError> {
    state
        .features
        .farmer
        .remove_farmer_use_case
        .execute(&staff_context, farmer_id(&id)?)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
