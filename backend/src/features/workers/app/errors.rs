use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::workers::domain::WorkerError,
    shared::DomainError,
};

impl ToErrorInfo for WorkerError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            WorkerError::BadName(_)
            | WorkerError::BadCost { .. }
            | WorkerError::NoteTooLong(_)
            | WorkerError::UnknownCostPer => {
                ErrorInfo::new(ErrorKind::InvalidInput, self.to_string())
            }
            WorkerError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Worker(#[from] WorkerError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Worker(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_broken_rule_of_a_card_is_invalid_input() {
        for error in [
            WorkerError::BadName(80),
            WorkerError::BadCost {
                min: 1_000,
                max: 10_000_000,
            },
            WorkerError::NoteTooLong(200),
            WorkerError::UnknownCostPer,
        ] {
            let info = error.to_error_info();

            assert_eq!(info.kind, ErrorKind::InvalidInput, "{error:?}");
            assert_eq!(info.code, "invalid", "{error:?}");
        }
    }
}
