use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::jobs::domain::JobError,
    shared::DomainError,
};

impl ToErrorInfo for JobError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            JobError::FinishesBeforeItStarts
            | JobError::StartsInTheFuture
            | JobError::TooOld { .. }
            | JobError::HalfFinished
            | JobError::NegativeRows => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "bad_run", self.to_string())
            }
            JobError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Job(#[from] JobError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),

    #[error("There is no job called {0}")]
    JobNotFound(String),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Job(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
            AppError::JobNotFound(_) => ErrorInfo::new(ErrorKind::NotFound, self.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_run_that_cannot_be_has_its_own_code_for_the_data_job() {
        for error in [
            JobError::FinishesBeforeItStarts,
            JobError::StartsInTheFuture,
            JobError::TooOld { days: 60 },
            JobError::HalfFinished,
            JobError::NegativeRows,
        ] {
            let info = error.to_error_info();

            assert_eq!(info.kind, ErrorKind::InvalidInput);
            assert_eq!(info.code, "bad_run");
        }
    }

    #[test]
    fn an_unknown_job_is_not_found_rather_than_invalid() {
        let info = AppError::JobNotFound("floods".to_string()).to_error_info();

        assert_eq!(info.kind, ErrorKind::NotFound);
        assert_eq!(info.code, "not_found");
    }
}
