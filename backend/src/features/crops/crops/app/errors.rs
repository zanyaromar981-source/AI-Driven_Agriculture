use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::crops::domain::CropError,
    shared::DomainError,
};

impl ToErrorInfo for CropError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            CropError::ReservedCode(_) => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "reserved_code", self.to_string())
            }
            CropError::AlreadyExists => {
                ErrorInfo::with_code(ErrorKind::Conflict, "already_exists", self.to_string())
            }
            CropError::InUse => {
                ErrorInfo::with_code(ErrorKind::Conflict, "crop_in_use", self.to_string())
            }
            CropError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Crop(#[from] CropError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Crop(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_crop_in_use_has_the_code_the_site_acts_on() {
        let info = CropError::InUse.to_error_info();

        assert_eq!(info.kind, ErrorKind::Conflict);
        assert_eq!(info.code, "crop_in_use");
    }

    #[test]
    fn a_taken_code_is_a_conflict() {
        let info = CropError::AlreadyExists.to_error_info();

        assert_eq!(info.kind, ErrorKind::Conflict);
        assert_eq!(info.code, "already_exists");
    }

    #[test]
    fn the_reserved_word_is_invalid_input_that_names_the_word() {
        let info = CropError::ReservedCode("empty".to_string()).to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "reserved_code");
        assert!(info.detail.contains("empty"), "{}", info.detail);
    }
}
