use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ToErrorInfo},
    features::alerts::domain::AlertError,
    shared::DomainError,
};

impl ToErrorInfo for AlertError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AlertError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Alert(#[from] AlertError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Alert(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::ErrorKind;

    #[test]
    fn a_bad_value_in_an_alert_is_invalid_input() {
        let error = AppError::from(AlertError::from(DomainError::InvalidValue(
            "Days must be 1 to 90".to_string(),
        )));

        assert_eq!(error.to_error_info().kind, ErrorKind::InvalidInput);
    }

    #[test]
    fn a_missing_alert_is_not_found() {
        let error = AppError::from(GlobalAppError::NotFound);

        assert_eq!(error.to_error_info().kind, ErrorKind::NotFound);
    }
}
