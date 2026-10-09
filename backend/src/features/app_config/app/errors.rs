use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::app_config::domain::AppConfigError,
    shared::DomainError,
};

impl ToErrorInfo for AppConfigError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppConfigError::BadVersion(_) => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "bad_version", self.to_string())
            }
            AppConfigError::MinAboveLatest { .. } => ErrorInfo::with_code(
                ErrorKind::InvalidInput,
                "min_above_latest",
                self.to_string(),
            ),
            AppConfigError::BadLimits(_) => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "bad_limits", self.to_string())
            }
            AppConfigError::BadMaintenanceWindow => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "bad_window", self.to_string())
            }
            AppConfigError::MissingText(_) => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "missing_text", self.to_string())
            }
            AppConfigError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    AppConfig(#[from] AppConfigError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),

    /// The app that called is older than the oldest version still allowed.
    #[error("This version of the app is no longer supported: update to {min_version} or newer")]
    UpdateRequired { min_version: String },
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::AppConfig(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
            AppError::UpdateRequired { .. } => {
                ErrorInfo::new(ErrorKind::UpgradeRequired, self.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_old_app_is_told_to_update_with_the_code_it_reads() {
        let info = AppError::UpdateRequired {
            min_version: "1.2.0".to_string(),
        }
        .to_error_info();

        assert_eq!(info.kind, ErrorKind::UpgradeRequired);
        assert_eq!(info.code, "update_required");
        assert!(info.detail.contains("1.2.0"), "{}", info.detail);
    }

    #[test]
    fn every_rule_a_saved_config_breaks_is_invalid_input_with_its_own_code() {
        for (error, code) in [
            (AppConfigError::BadVersion("x".to_string()), "bad_version"),
            (
                AppConfigError::MinAboveLatest {
                    min: "2.0.0".to_string(),
                    latest: "1.0.0".to_string(),
                },
                "min_above_latest",
            ),
            (AppConfigError::BadLimits("x".to_string()), "bad_limits"),
            (AppConfigError::BadMaintenanceWindow, "bad_window"),
            (
                AppConfigError::MissingText("announcement_ku"),
                "missing_text",
            ),
        ] {
            let info = error.to_error_info();

            assert_eq!(info.kind, ErrorKind::InvalidInput, "{error:?}");
            assert_eq!(info.code, code, "{error:?}");
        }
    }
}
