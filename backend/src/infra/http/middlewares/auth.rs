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
    app::{AppError, AuthContext, ErrorKind, ToErrorInfo, User},
    shared::{AppState, JwtClaims, JwtError, Phone},
};

impl TryFrom<JwtClaims> for User {
    type Error = AppError;
    fn try_from(claims: JwtClaims) -> Result<Self, Self::Error> {
        Ok(Self {
            phone: Phone::new(claims.sub)
                .map_err(|_e| AppError::JwtError(JwtError::InvalidSubject))?,
        })
    }
}

/// Guards the farmer routes. It accepts only a token issued for the app,
/// then asks the farmers feature whether that farmer still exists, on every
/// request, so the token of a farmer staff have removed stops working at
/// once.
pub async fn auth(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let (mut parts, body) = req.into_parts();

    // Extract authorization header with proper error handling
    let TypedHeader(Authorization(bearer)) =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(header) => header,
            Err(_) => {
                return Err(AppError::Unauthorized(
                    JwtError::MissingAuthorizationHeader.to_string(),
                ));
            }
        };

    let token = bearer.token();

    match crate::shared::validate_jwt(
        token,
        &state.config.auth.jwt_secret,
        &state.config.auth.issuer,
        &state.config.auth.audience,
    ) {
        Ok(claims) => {
            let user = User::try_from(claims).map_err(|e| AppError::Unauthorized(e.to_string()))?;

            state
                .features
                .farmer
                .identify_farmer_use_case
                .execute(user.phone())
                .await
                .map_err(|error| {
                    let info = error.to_error_info();

                    // A fault of ours (the database is down) stays a server
                    // error: the app signs the farmer out on 401. Everything
                    // else, whatever the reason, is the same 401.
                    if matches!(info.kind, ErrorKind::Persistence | ErrorKind::Internal) {
                        AppError::InternalServerError
                    } else {
                        AppError::Unauthorized(info.detail)
                    }
                })?;

            // Reconstruct the request with the claims in extensions
            let mut req = Request::from_parts(parts, body);

            let auth_context = AuthContext::new(user, token.to_string());

            req.extensions_mut().insert(auth_context);

            Ok(next.run(req).await)
        }
        Err(e) => Err(AppError::Unauthorized(e.to_string())),
    }
}
