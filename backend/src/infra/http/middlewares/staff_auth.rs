use axum::{
    extract::{FromRequestParts, Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};

use crate::{
    app::{AppError, ErrorKind, StaffContext, ToErrorInfo},
    shared::{AppState, JwtError},
};

/// Guards the dashboard. It accepts only a token issued for the dashboard
/// audience, then reads the staff member and their permissions again on
/// every request, so a role change or a deactivation takes effect at once.
/// The `StaffContext` it puts on the request is what `check_permission`
/// reads.
pub async fn staff_auth(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let (mut parts, body) = req.into_parts();

    let TypedHeader(Authorization(bearer)) =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(header) => header,
            Err(_) => {
                return Err(AppError::Unauthorized(
                    JwtError::MissingAuthorizationHeader.to_string(),
                ));
            }
        };

    let claims = crate::shared::validate_jwt(
        bearer.token(),
        &state.config.auth.jwt_secret,
        &state.config.auth.issuer,
        &state.config.auth.dashboard_audience,
    )
    .map_err(|error| AppError::Unauthorized(error.to_string()))?;

    // A dashboard token carries the staff id. Anything else is not ours.
    let staff_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Unauthorized(JwtError::InvalidSubject.to_string()))?;

    let access = state
        .features
        .staff
        .identify_staff_use_case
        .execute(staff_id)
        .await
        .map_err(|error| {
            let info = error.to_error_info();

            // A fault of ours (the database is down) stays a server error:
            // answering 401 would sign every dashboard out. Everything
            // else, whatever the reason, is the same 401.
            if matches!(info.kind, ErrorKind::Persistence | ErrorKind::Internal) {
                AppError::InternalServerError
            } else {
                AppError::Unauthorized(info.detail)
            }
        })?;

    let staff_context = StaffContext::new(
        staff_id,
        access.staff().email().into(),
        access.permissions().iter().copied().collect(),
    );

    let mut req = Request::from_parts(parts, body);

    req.extensions_mut().insert(staff_context);

    Ok(next.run(req).await)
}
