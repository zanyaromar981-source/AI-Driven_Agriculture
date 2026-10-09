use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::rules::domain::RuleError,
    shared::DomainError,
};

impl ToErrorInfo for RuleError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            RuleError::ValueOutOfRange { .. } | RuleError::ValueNotANumber => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "bad_range", self.to_string())
            }
            RuleError::ReasonLength { .. } => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "bad_reason", self.to_string())
            }
            RuleError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Rule(#[from] RuleError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),

    #[error("There is no rule called {0}")]
    RuleNotFound(String),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Rule(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
            AppError::RuleNotFound(_) => ErrorInfo::new(ErrorKind::NotFound, self.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_value_outside_the_range_has_the_code_the_site_acts_on() {
        let info = RuleError::ValueOutOfRange {
            code: "frost_c".to_string(),
            value: 9.0,
            min: -1.0,
            max: 5.0,
        }
        .to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "bad_range");
        assert!(
            info.detail.contains("-1") && info.detail.contains('5'),
            "the detail names the range: {}",
            info.detail
        );
    }

    #[test]
    fn a_bad_reason_has_its_own_code() {
        let info = RuleError::ReasonLength { min: 3, max: 500 }.to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "bad_reason");
    }

    #[test]
    fn an_unknown_rule_is_not_found_rather_than_invalid() {
        let info = AppError::RuleNotFound("frost_f".to_string()).to_error_info();

        assert_eq!(info.kind, ErrorKind::NotFound);
        assert_eq!(info.code, "not_found");
    }
}
