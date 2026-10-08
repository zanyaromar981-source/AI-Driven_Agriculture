use axum::{
    Json,
    extract::{FromRequest, Request, rejection::JsonRejection},
    response::{IntoResponse, Response},
};
use validator::Validate;

use super::errors::HttpErrorResponse;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: serde::de::DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ValidationRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(ValidationRejection::JsonRejection)?;

        value
            .validate()
            .map_err(ValidationRejection::ValidationError)?;

        Ok(ValidatedJson(value))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationRejection {
    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),
}

impl IntoResponse for ValidationRejection {
    fn into_response(self) -> Response {
        match self {
            ValidationRejection::JsonRejection(rejection) => {
                HttpErrorResponse::bad_request(rejection.to_string()).into_response()
            }
            ValidationRejection::ValidationError(errors) => {
                // The body carries one `field`, so the first failing field is
                // the one reported.
                let (field, detail) = errors
                    .field_errors()
                    .into_iter()
                    .next()
                    .map(|(field, errors)| {
                        let detail = errors
                            .first()
                            .and_then(|error| error.message.as_ref())
                            .map(|message| message.to_string())
                            .unwrap_or_else(|| "Invalid value".to_string());

                        (field.to_string(), detail)
                    })
                    .unwrap_or_else(|| ("body".to_string(), "Invalid value".to_string()));

                HttpErrorResponse::invalid_field(field, detail).into_response()
            }
        }
    }
}
