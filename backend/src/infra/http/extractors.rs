use axum::{
    Json,
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use validator::Validate;

use super::errors::{ErrorWrapper, FieldError};

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
        let (status, error_wrapper) = match self {
            ValidationRejection::JsonRejection(rejection) => (
                StatusCode::BAD_REQUEST,
                ErrorWrapper::single("global", "Invalid Request Body", rejection.to_string()),
            ),
            ValidationRejection::ValidationError(errors) => {
                let field_errors: Vec<FieldError> = errors
                    .field_errors()
                    .into_iter()
                    .flat_map(|(field, errors)| {
                        errors.iter().map(move |error| FieldError {
                            field: field.to_string(),
                            title: "Validation Error".to_string(),
                            detail: error
                                .message
                                .as_ref()
                                .map(|m| m.to_string())
                                .unwrap_or_else(|| "Invalid value".to_string()),
                        })
                    })
                    .collect();

                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ErrorWrapper::new(field_errors),
                )
            }
        };

        (status, Json(error_wrapper)).into_response()
    }
}
