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
            | FarmError::OutlineEnclosesNoCells => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "bad_polygon", self.to_string())
            }
            FarmError::TooManyCells(_) => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "farm_too_large", self.to_string())
            }
            FarmError::UnknownCrop(_) => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "unknown_crop", self.to_string())
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
                ErrorInfo::with_code(ErrorKind::InvalidInput, "too_many_farms", self.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_crop_that_cannot_be_used_is_invalid_input_with_its_own_code_and_its_name() {
        let info = FarmError::UnknownCrop("rice".to_string()).to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "unknown_crop");
        assert!(info.detail.contains("rice"), "{}", info.detail);
    }
}
