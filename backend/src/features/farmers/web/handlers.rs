use axum::{Extension, extract::State};

use super::{
    dtos::{
        EditProfileParams, ProfileResponse, SendSignInCodeParams, SignInCodeSentResponse,
        SignedInResponse, VerifySignInCodeParams,
    },
    errors::WebError,
};

use crate::{
    app::AuthContext,
    infra::http::{ApiResponse, ErrorBody, ValidatedJson},
    shared::AppState,
};

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
