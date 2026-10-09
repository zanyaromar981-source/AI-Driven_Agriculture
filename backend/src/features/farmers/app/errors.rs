use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::farmers::domain::FarmerError,
    shared::DomainError,
};

impl ToErrorInfo for FarmerError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            // One code for every way a code can fail: the app shows the same
            // "wrong code" screen, and an attacker learns nothing extra.
            FarmerError::WrongCode | FarmerError::CodeExpired => {
                ErrorInfo::with_code(ErrorKind::Authentication, "bad_code", self.to_string())
            }
            FarmerError::CodeRequestedTooSoon(seconds) => {
                ErrorInfo::new(ErrorKind::RateLimited, self.to_string()).retry_after(*seconds)
            }
            FarmerError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Farmer(#[from] FarmerError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),

    #[error("A farmer with this phone already exists")]
    FarmerAlreadyExists,
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Farmer(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
            AppError::FarmerAlreadyExists => {
                ErrorInfo::with_code(ErrorKind::Conflict, "already_exists", self.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_failed_code_looks_the_same_to_the_caller() {
        for error in [FarmerError::WrongCode, FarmerError::CodeExpired] {
            let info = error.to_error_info();

            assert_eq!(info.kind, ErrorKind::Authentication);
            assert_eq!(info.code, "bad_code");
        }
    }

    #[test]
    fn a_phone_that_already_has_a_farmer_is_a_conflict_the_dashboard_can_read() {
        let info = AppError::FarmerAlreadyExists.to_error_info();

        assert_eq!(info.kind, ErrorKind::Conflict);
        assert_eq!(info.code, "already_exists");
    }

    #[test]
    fn asking_too_soon_is_a_rate_limit_with_the_wait() {
        let info = FarmerError::CodeRequestedTooSoon(42).to_error_info();

        assert_eq!(info.kind, ErrorKind::RateLimited);
        assert_eq!(info.retry_after_s, Some(42));
    }
}
