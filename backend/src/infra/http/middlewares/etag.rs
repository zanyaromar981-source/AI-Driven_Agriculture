use axum::{
    body::{Body, to_bytes},
    extract::Request,
    http::{HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::Response,
};
use sha2::{Digest, Sha256};

/// Changes whenever the shape of any answer changes. A website that caches
/// answers throws its cache away when it sees a new value.
pub const API_VERSION: &str = "1.7.0";

const API_VERSION_HEADER: &str = "x-api-version";

/// Bodies above this size are passed through without a tag: hashing them
/// would mean holding them in memory twice.
const MAX_TAGGED_BODY: usize = 8 * 1024 * 1024;

/// Lets a client keep its copy when nothing changed.
///
/// Every successful JSON `GET` answer gets an `ETag` made from its own bytes.
/// A client that sends that tag back in `If-None-Match` gets `304` with no
/// body when the answer would be the same. The tag is made from the answer,
/// not from a version counter, so it is right for every route without a list
/// of which route reads which data; the cost is that the answer is still
/// worked out before it is thrown away. `GET /v1/versions` is the cheap way
/// to ask "did anything change" first.
pub async fn etag(req: Request, next: Next) -> Response {
    let is_get = req.method() == Method::GET;
    let has_token = req.headers().contains_key(header::AUTHORIZATION);
    let known = req
        .headers()
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    let mut response = next.run(req).await;

    response
        .headers_mut()
        .insert(API_VERSION_HEADER, HeaderValue::from_static(API_VERSION));

    let is_json = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|kind| kind.starts_with("application/json"));

    if !is_get || response.status() != StatusCode::OK || !is_json {
        return response;
    }

    let (mut parts, body) = response.into_parts();

    let Ok(bytes) = to_bytes(body, MAX_TAGGED_BODY).await else {
        // Too large to tag, and the body is spent: answer plainly rather
        // than send half of it.
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap_or_default();
    };

    let tag = tag_of(&bytes);

    if let Ok(value) = HeaderValue::from_str(&tag) {
        parts.headers.insert(header::ETAG, value);
    }
    // "no-cache" means "ask before reusing", which is what the tag is for.
    // An answer made for a token must not be kept by a shared cache.
    parts.headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(if has_token {
            "private, no-cache"
        } else {
            "no-cache"
        }),
    );

    if known.is_some_and(|known| matches(&known, &tag)) {
        parts.status = StatusCode::NOT_MODIFIED;
        parts.headers.remove(header::CONTENT_LENGTH);
        parts.headers.remove(header::CONTENT_TYPE);

        return Response::from_parts(parts, Body::empty());
    }

    Response::from_parts(parts, Body::from(bytes))
}

fn tag_of(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);

    format!("W/\"{}\"", hex::encode(&digest[..10]))
}

/// `If-None-Match` may list several tags, or be `*`.
fn matches(known: &str, tag: &str) -> bool {
    let bare = |value: &str| value.trim().trim_start_matches("W/").to_string();

    known.trim() == "*" || known.split(',').any(|one| bare(one) == bare(tag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_bytes_always_get_the_same_tag() {
        assert_eq!(tag_of(b"{\"farms\":[]}"), tag_of(b"{\"farms\":[]}"));
    }

    #[test]
    fn different_bytes_get_different_tags() {
        assert_ne!(tag_of(b"{\"farms\":[]}"), tag_of(b"{\"farms\":[1]}"));
    }

    #[test]
    fn the_tag_is_a_weak_quoted_tag() {
        let tag = tag_of(b"x");

        assert!(tag.starts_with("W/\""));
        assert!(tag.ends_with('"'));
    }

    #[test]
    fn a_known_tag_matches_with_or_without_the_weak_mark() {
        let tag = tag_of(b"x");
        let strong = tag.trim_start_matches("W/").to_string();

        assert!(matches(&tag, &tag));
        assert!(matches(&strong, &tag));
    }

    #[test]
    fn one_of_several_known_tags_is_enough() {
        let tag = tag_of(b"x");

        assert!(matches(&format!("W/\"old\", {tag}"), &tag));
        assert!(!matches("W/\"old\", W/\"older\"", &tag));
    }

    #[test]
    fn a_star_matches_anything() {
        assert!(matches("*", &tag_of(b"x")));
    }
}
