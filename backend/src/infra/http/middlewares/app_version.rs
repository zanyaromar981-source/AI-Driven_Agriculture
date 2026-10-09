use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{app::AuthContext, infra::http::HttpErrorResponse, shared::AppState};

pub const APP_VERSION_HEADER: &str = "x-app-version";

/// Refuses a farmer app that is older than the oldest version staff still
/// allow, with `426 update_required`. A request with no `X-App-Version`, or
/// one that is not a version, goes through: older builds and tools like curl
/// send none.
///
/// It sits on the farmer routes, inside the `auth` layer so that it knows
/// which farmer is asking and can note the version in use, and on the
/// sign-in routes, where there is no farmer yet. `GET /v1/app/config` is
/// outside it on purpose: that is where an old app learns it must update.
pub async fn app_version(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, HttpErrorResponse> {
    let declared = req
        .headers()
        .get(APP_VERSION_HEADER)
        .and_then(|value| value.to_str().ok());

    let phone = req
        .extensions()
        .get::<AuthContext>()
        .map(|auth_context| auth_context.user().phone());

    state
        .features
        .app_config
        .check_app_version_use_case
        .execute(declared, phone)
        .await
        .map_err(|error| HttpErrorResponse::from_error(&error))?;

    Ok(next.run(req).await)
}
