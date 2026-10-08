use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::fires::domain::FireError,
    shared::DomainError,
};

impl ToErrorInfo for FireError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            FireError::OutsideRegion => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "outside_region", self.to_string())
            }
            FireError::WindowOutOfRange { .. } => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "bad_window", self.to_string())
            }
            FireError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Fire(#[from] FireError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Fire(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fire_outside_the_region_has_a_code_the_job_can_act_on() {
        let info = FireError::OutsideRegion.to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "outside_region");
    }

    #[test]
    fn a_window_out_of_range_has_its_own_code() {
        let info = FireError::WindowOutOfRange { min: 1, max: 168 }.to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "bad_window");
    }
}
