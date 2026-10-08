use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::insights::domain::InsightError,
    shared::DomainError,
};

impl ToErrorInfo for InsightError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            InsightError::MeasureCount { .. } => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "bad_measures", self.to_string())
            }
            InsightError::DuplicateMeasureCode(_) => ErrorInfo::with_code(
                ErrorKind::InvalidInput,
                "duplicate_measure",
                self.to_string(),
            ),
            InsightError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Insight(#[from] InsightError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Insight(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wrong_number_of_measures_has_a_code_the_job_can_act_on() {
        let info = InsightError::MeasureCount { min: 1, max: 20 }.to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "bad_measures");
    }

    #[test]
    fn a_repeated_measure_code_is_invalid_input_not_a_conflict() {
        let info = InsightError::DuplicateMeasureCode("level_pct".to_string()).to_error_info();

        assert_eq!(
            info.kind,
            ErrorKind::InvalidInput,
            "the clash is inside one request body, not with stored data"
        );
        assert_eq!(info.code, "duplicate_measure");
        assert!(info.detail.contains("level_pct"));
    }
}
