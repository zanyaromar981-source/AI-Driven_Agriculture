use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        CreateStaffParams, SaveStaffRoleParams, StaffListResponse, StaffLoginParams,
        StaffMeResponse, StaffOneResponse, StaffOneRoleResponse, StaffPermissionCatalogueResponse,
        StaffResponse, StaffRoleResponse, StaffRolesResponse, StaffSignedInResponse,
        UpdateStaffParams,
    },
    errors::WebError,
};

use crate::{
    app::StaffContext,
    infra::http::{ApiResponse, ErrorBody, ValidatedJson},
    shared::AppState,
};

/// Ids travel as opaque strings. One that is not a number cannot name a
/// role or a staff member, so it is not found rather than a bad request.
fn numeric_id(raw: &str) -> Result<i32, WebError> {
    raw.parse().map_err(|_| WebError::not_found())
}

/// Sign in to the dashboard with an email and a password
#[utoipa::path(
    post,
    path = "/v1/dashboard/auth/login",
    tag = "staff",
    request_body = StaffLoginParams,
    responses(
        (status = 200, description = "Signed in", body = StaffSignedInResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Wrong email or password, or an inactive account", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn sign_in(
    State(state): State<AppState>,
    ValidatedJson(params): ValidatedJson<StaffLoginParams>,
) -> Result<ApiResponse<StaffSignedInResponse>, WebError> {
    let input = params.into_input()?;

    let signed_in = state.features.staff.sign_in_use_case.execute(input).await?;

    Ok(ApiResponse::ok(StaffSignedInResponse::from(&signed_in)))
}

/// Get the signed-in staff member and what they may do
#[utoipa::path(
    get,
    path = "/v1/dashboard/me",
    tag = "staff",
    responses(
        (status = 200, description = "Staff member retrieved successfully", body = StaffMeResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_me(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
) -> Result<ApiResponse<StaffMeResponse>, WebError> {
    let staff = state
        .features
        .staff
        .view_staff_use_case
        .execute(*staff_context.staff_id())
        .await?;

    Ok(ApiResponse::ok(StaffMeResponse::from((
        &staff,
        &staff_context,
    ))))
}

/// List the resources and actions a role can be built from
#[utoipa::path(
    get,
    path = "/v1/dashboard/permissions",
    tag = "staff",
    responses(
        (status = 200, description = "Catalogue retrieved successfully", body = StaffPermissionCatalogueResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_permission_catalogue() -> ApiResponse<StaffPermissionCatalogueResponse> {
    ApiResponse::ok(StaffPermissionCatalogueResponse::everything())
}

/// List all roles
#[utoipa::path(
    get,
    path = "/v1/dashboard/roles",
    tag = "staff",
    responses(
        (status = 200, description = "List of roles retrieved successfully", body = StaffRolesResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs roles:read", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_roles(
    State(state): State<AppState>,
) -> Result<ApiResponse<StaffRolesResponse>, WebError> {
    let roles = state.features.staff.list_roles_use_case.execute().await?;

    Ok(ApiResponse::ok(StaffRolesResponse {
        roles: roles.iter().map(StaffRoleResponse::from).collect(),
    }))
}

/// Create a custom role
#[utoipa::path(
    post,
    path = "/v1/dashboard/roles",
    tag = "staff",
    request_body = SaveStaffRoleParams,
    responses(
        (status = 201, description = "Role created successfully", body = StaffOneRoleResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs roles:create", body = ErrorBody),
        (status = 409, description = "Another role has this name (`role_name_taken`)", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_role(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    ValidatedJson(params): ValidatedJson<SaveStaffRoleParams>,
) -> Result<ApiResponse<StaffOneRoleResponse>, WebError> {
    let input = params.into_create_input()?;

    let role = state
        .features
        .staff
        .create_role_use_case
        .execute(&staff_context, input)
        .await?;

    Ok(ApiResponse::created(StaffOneRoleResponse {
        role: StaffRoleResponse::from(&role),
    }))
}

/// Get one role
#[utoipa::path(
    get,
    path = "/v1/dashboard/roles/{id}",
    tag = "staff",
    params(("id" = String, Path, description = "Role ID")),
    responses(
        (status = 200, description = "Role retrieved successfully", body = StaffOneRoleResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs roles:read", body = ErrorBody),
        (status = 404, description = "Role not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_role(
    State(state): State<AppState>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<ApiResponse<StaffOneRoleResponse>, WebError> {
    let role = state
        .features
        .staff
        .view_role_use_case
        .execute(numeric_id(&id)?)
        .await?;

    Ok(ApiResponse::ok(StaffOneRoleResponse {
        role: StaffRoleResponse::from(&role),
    }))
}

/// Replace a role's name, description and whole permission set
#[utoipa::path(
    put,
    path = "/v1/dashboard/roles/{id}",
    tag = "staff",
    params(("id" = String, Path, description = "Role ID")),
    request_body = SaveStaffRoleParams,
    responses(
        (status = 200, description = "Role updated successfully", body = StaffOneRoleResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs roles:update", body = ErrorBody),
        (status = 404, description = "Role not found", body = ErrorBody),
        (status = 409, description = "A system role (`system_role`), or another role has this name (`role_name_taken`)", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_role(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<SaveStaffRoleParams>,
) -> Result<ApiResponse<StaffOneRoleResponse>, WebError> {
    let input = params.into_edit_input()?;

    let role = state
        .features
        .staff
        .edit_role_use_case
        .execute(&staff_context, numeric_id(&id)?, input)
        .await?;

    Ok(ApiResponse::ok(StaffOneRoleResponse {
        role: StaffRoleResponse::from(&role),
    }))
}

/// Delete a role nobody holds
#[utoipa::path(
    delete,
    path = "/v1/dashboard/roles/{id}",
    tag = "staff",
    params(("id" = String, Path, description = "Role ID")),
    responses(
        (status = 204, description = "Delete was successful"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs roles:delete", body = ErrorBody),
        (status = 404, description = "Role not found", body = ErrorBody),
        (status = 409, description = "A system role (`system_role`), or still assigned (`role_in_use`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_role(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<StatusCode, WebError> {
    state
        .features
        .staff
        .delete_role_use_case
        .execute(&staff_context, numeric_id(&id)?)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// List all staff members
#[utoipa::path(
    get,
    path = "/v1/dashboard/staff",
    tag = "staff",
    responses(
        (status = 200, description = "List of staff retrieved successfully", body = StaffListResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs staff:read", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_staff_list(
    State(state): State<AppState>,
) -> Result<ApiResponse<StaffListResponse>, WebError> {
    let staff = state.features.staff.list_staff_use_case.execute().await?;

    Ok(ApiResponse::ok(StaffListResponse {
        staff: staff.iter().map(StaffResponse::from).collect(),
    }))
}

/// Add a staff member
#[utoipa::path(
    post,
    path = "/v1/dashboard/staff",
    tag = "staff",
    request_body = CreateStaffParams,
    responses(
        (status = 201, description = "Staff member created successfully", body = StaffOneResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs staff:create", body = ErrorBody),
        (status = 409, description = "Another account has this email (`email_taken`)", body = ErrorBody),
        (status = 422, description = "Validation error, or a role that does not exist (`unknown_role`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_staff(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    ValidatedJson(params): ValidatedJson<CreateStaffParams>,
) -> Result<ApiResponse<StaffOneResponse>, WebError> {
    let input = params.into_input()?;

    let staff = state
        .features
        .staff
        .add_staff_use_case
        .execute(&staff_context, input)
        .await?;

    Ok(ApiResponse::created(StaffOneResponse {
        staff: StaffResponse::from(&staff),
    }))
}

/// Get one staff member
#[utoipa::path(
    get,
    path = "/v1/dashboard/staff/{id}",
    tag = "staff",
    params(("id" = String, Path, description = "Staff ID")),
    responses(
        (status = 200, description = "Staff member retrieved successfully", body = StaffOneResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs staff:read", body = ErrorBody),
        (status = 404, description = "Staff member not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_staff(
    State(state): State<AppState>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<ApiResponse<StaffOneResponse>, WebError> {
    let staff = state
        .features
        .staff
        .view_staff_use_case
        .execute(numeric_id(&id)?)
        .await?;

    Ok(ApiResponse::ok(StaffOneResponse {
        staff: StaffResponse::from(&staff),
    }))
}

/// Change a staff member's name, state, roles and, optionally, password
#[utoipa::path(
    put,
    path = "/v1/dashboard/staff/{id}",
    tag = "staff",
    params(("id" = String, Path, description = "Staff ID")),
    request_body = UpdateStaffParams,
    responses(
        (status = 200, description = "Staff member updated successfully", body = StaffOneResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs staff:update", body = ErrorBody),
        (status = 404, description = "Staff member not found", body = ErrorBody),
        (status = 409, description = "Your own account (`own_account`), or the last active owner (`last_owner`)", body = ErrorBody),
        (status = 422, description = "Validation error, or a role that does not exist (`unknown_role`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_staff(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<UpdateStaffParams>,
) -> Result<ApiResponse<StaffOneResponse>, WebError> {
    let input = params.into_input()?;

    let staff = state
        .features
        .staff
        .edit_staff_use_case
        .execute(&staff_context, numeric_id(&id)?, input)
        .await?;

    Ok(ApiResponse::ok(StaffOneResponse {
        staff: StaffResponse::from(&staff),
    }))
}

/// Delete a staff member
#[utoipa::path(
    delete,
    path = "/v1/dashboard/staff/{id}",
    tag = "staff",
    params(("id" = String, Path, description = "Staff ID")),
    responses(
        (status = 204, description = "Delete was successful"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs staff:delete", body = ErrorBody),
        (status = 404, description = "Staff member not found", body = ErrorBody),
        (status = 409, description = "Your own account (`own_account`), or the last active owner (`last_owner`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_staff(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<StatusCode, WebError> {
    state
        .features
        .staff
        .remove_staff_use_case
        .execute(&staff_context, numeric_id(&id)?)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
