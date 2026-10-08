use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{app::AppError, shared::AppState};

pub const SERVICE_KEY_HEADER: &str = "x-service-key";

/// Guards the routes the data jobs write through. The jobs are our own
/// scripts, not farmers, so they prove themselves with one shared key
/// instead of a sign-in token.
pub async fn service_key(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let presented = req
        .headers()
        .get(SERVICE_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    if !key_matches(state.config.ingest.service_key.as_deref(), presented) {
        return Err(AppError::Unauthorized(
            "Missing or wrong service key".to_string(),
        ));
    }

    Ok(next.run(req).await)
}

/// With no key configured nothing matches, so a server started without one
/// cannot be written to by anyone.
fn key_matches(configured: Option<&str>, presented: &str) -> bool {
    let Some(configured) = configured else {
        return false;
    };

    if configured.len() != presented.len() {
        return false;
    }

    // Compare every byte so the time taken does not show where they differ.
    configured
        .bytes()
        .zip(presented.bytes())
        .fold(0u8, |difference, (left, right)| difference | (left ^ right))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_configured_key_is_accepted() {
        assert!(key_matches(Some("long-random-key"), "long-random-key"));
    }

    #[test]
    fn another_key_is_refused() {
        assert!(!key_matches(Some("long-random-key"), "long-random-kez"));
        assert!(!key_matches(Some("long-random-key"), "long-random"));
        assert!(!key_matches(Some("long-random-key"), ""));
    }

    #[test]
    fn a_server_without_a_key_accepts_nothing() {
        assert!(!key_matches(None, ""));
        assert!(!key_matches(None, "anything"));
    }
}
