use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// A JSON answer with its status. The body is sent as it is, with no outer
/// wrapper, because the app reads the shapes in `BACKEND.md` directly.
pub struct ApiResponse<T: Serialize> {
    status_code: StatusCode,
    body: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(body: T) -> Self {
        Self {
            status_code: StatusCode::OK,
            body,
        }
    }

    pub fn created(body: T) -> Self {
        Self {
            status_code: StatusCode::CREATED,
            body,
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        (self.status_code, Json(self.body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_status_travels_in_the_http_status_never_the_body() {
        let response = ApiResponse::created(serde_json::json!({"farm": {"id": "1"}}));

        assert_eq!(response.status_code, StatusCode::CREATED);
        assert!(
            serde_json::to_value(&response.body)
                .expect("body serializes")
                .get("status_code")
                .is_none()
        );
    }

    #[test]
    fn ok_and_created_differ_only_in_status() {
        assert_eq!(ApiResponse::ok(1).status_code, StatusCode::OK);
        assert_eq!(ApiResponse::created(1).status_code, StatusCode::CREATED);
    }
}
