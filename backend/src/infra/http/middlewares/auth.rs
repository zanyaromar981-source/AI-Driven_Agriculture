use axum::{
    extract::{FromRequestParts, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};

use crate::{
    app::{AppError, AuthContext, ErrorKind, ToErrorInfo, User},
    infra::http::HttpErrorResponse,
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

/// The farmer who is asking, on a route that also answers without a login
/// and shows a signed-in farmer more. The token is checked exactly as
/// `auth` checks it; one that is missing, wrong, expired, not the app's, or
/// of a farmer who was removed or blocked is no token at all, so the answer
/// is the public one rather than a refusal.
pub struct OptionalAuth(pub Option<AuthContext>);

impl FromRequestParts<AppState> for OptionalAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Ok(TypedHeader(Authorization(bearer))) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await
        else {
            return Ok(Self(None));
        };

        let Ok(claims) = crate::shared::validate_jwt(
            bearer.token(),
            &state.config.auth.jwt_secret,
            &state.config.auth.issuer,
            &state.config.auth.audience,
        ) else {
            return Ok(Self(None));
        };

        let Ok(user) = User::try_from(claims) else {
            return Ok(Self(None));
        };

        match state
            .features
            .farmer
            .identify_farmer_use_case
            .execute(user.phone())
            .await
        {
            Ok(_) => Ok(Self(Some(AuthContext::new(
                user,
                bearer.token().to_string(),
            )))),
            Err(error) => match error.to_error_info().kind {
                // A fault of ours is not a reason to quietly hide what the
                // farmer may see.
                ErrorKind::Persistence | ErrorKind::Internal => Err(AppError::InternalServerError),
                _ => Ok(Self(None)),
            },
        }
    }
}

/// Guards the farmer routes. It accepts only a token issued for the app,
/// then asks the farmers feature whether that farmer still exists and is
/// still let in, on every request, so the token of a farmer staff have
/// removed or blocked stops working at once.
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

            if let Err(error) = state
                .features
                .farmer
                .identify_farmer_use_case
                .execute(user.phone())
                .await
            {
                let info = error.to_error_info();

                return match info.kind {
                    // A fault of ours (the database is down) stays a server
                    // error: the app signs the farmer out on 401.
                    ErrorKind::Persistence | ErrorKind::Internal => {
                        Err(AppError::InternalServerError)
                    }
                    // A farmer staff have blocked is told so, with the code
                    // the farmers feature gave (`blocked`), so the app can
                    // say why instead of asking for a new sign-in code.
                    ErrorKind::Authorization => Ok(HttpErrorResponse::from(info).into_response()),
                    // Everything else, whatever the reason, is the same 401.
                    _ => Err(AppError::Unauthorized(info.detail)),
                };
            }

            // Reconstruct the request with the claims in extensions
            let mut req = Request::from_parts(parts, body);

            let auth_context = AuthContext::new(user, token.to_string());

            req.extensions_mut().insert(auth_context);

            Ok(next.run(req).await)
        }
        Err(e) => Err(AppError::Unauthorized(e.to_string())),
    }
}
