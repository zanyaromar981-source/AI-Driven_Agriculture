use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ResponseWrapper<T: Serialize> {
    #[serde(skip_serializing)]
    pub status_code: StatusCode,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
}

impl<T: Serialize> ResponseWrapper<T> {
    pub fn new(data: T, meta: Option<Meta>, status_code: StatusCode) -> Self {
        Self {
            status_code,
            data,
            meta,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Meta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
    pub rows_per_page: Option<u64>,
    pub page: Option<u64>,
}

impl<T: Serialize> IntoResponse for ResponseWrapper<T> {
    fn into_response(self) -> Response {
        (self.status_code, Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(count: Option<u64>) -> serde_json::Value {
        serde_json::to_value(Meta {
            count,
            rows_per_page: Some(20),
            page: Some(1),
        })
        .expect("meta serializes")
    }

    #[test]
    fn the_first_page_carries_the_count() {
        assert_eq!(meta(Some(42))["count"], 42);
    }

    #[test]
    fn a_later_page_omits_the_count_rather_than_sending_null() {
        let body = meta(None);

        assert!(
            body.get("count").is_none(),
            "a null count would look like a real value of unknown size to a client"
        );
        assert_eq!(body["page"], 1);
        assert_eq!(body["rows_per_page"], 20);
    }

    #[test]
    fn a_wrapper_without_meta_omits_the_key_entirely() {
        let body = serde_json::to_value(ResponseWrapper::new(vec![1, 2, 3], None, StatusCode::OK))
            .expect("wrapper serializes");

        assert!(body.get("meta").is_none());
        assert_eq!(body["data"], serde_json::json!([1, 2, 3]));
        assert!(
            body.get("status_code").is_none(),
            "the status code travels in the HTTP status, never the body"
        );
    }
}
