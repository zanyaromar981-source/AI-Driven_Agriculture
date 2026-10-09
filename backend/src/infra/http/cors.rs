use std::time::Duration;

use axum::http::{HeaderName, HeaderValue, Method, header};
use tower_http::cors::{AllowOrigin, CorsLayer};

/// Lets the listed websites call the API from a browser. Requests carry a
/// token, so origins are named one by one and never `*`. With no origin
/// configured nothing is allowed, which is right for a server that only the
/// phone app and the data jobs talk to.
pub fn cors_layer(origins: &[String]) -> CorsLayer {
    let allowed: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|origin| match HeaderValue::from_str(origin) {
            Ok(value) => Some(value),
            Err(_) => {
                tracing::warn!(%origin, "ignoring a CORS origin that is not a valid header value");
                None
            }
        })
        .collect();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::IF_NONE_MATCH,
            HeaderName::from_static("idempotency-key"),
            HeaderName::from_static("x-app-version"),
        ])
        .expose_headers([header::ETAG, HeaderName::from_static("x-api-version")])
        .max_age(Duration::from_secs(600))
}
