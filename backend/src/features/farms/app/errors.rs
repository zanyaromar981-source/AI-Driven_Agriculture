use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::farms::domain::FarmError,
    shared::DomainError,
};

impl ToErrorInfo for FarmError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            FarmError::OutlineCornerCount { .. }
            | FarmError::OutlineSelfIntersects
            | FarmError::OutlineEnclosesNoCells
            | FarmError::TooManyCells(_) => {
                ErrorInfo::new(ErrorKind::InvalidInput, self.to_string())
            }
            FarmError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Farm(#[from] FarmError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),

    #[error("You have reached the maximum number of farms per user ({0})")]
    MaxFarmsPerUserReached(u64),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Farm(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
            AppError::MaxFarmsPerUserReached(_) => {
                ErrorInfo::new(ErrorKind::InvalidInput, self.to_string())
            }
        }
    }
}
