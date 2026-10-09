use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::briefs::domain::BriefError,
    shared::DomainError,
};

impl ToErrorInfo for BriefError {
    fn to_error_info(&self) -> ErrorInfo {
        let invalid = |code| ErrorInfo::with_code(ErrorKind::InvalidInput, code, self.to_string());

        match self {
            BriefError::TooManyPoints { .. } => invalid("bad_points"),
            BriefError::TooManySources { .. } => invalid("bad_sources"),
            BriefError::FarmCount { .. } => invalid("bad_farms"),
            BriefError::DuplicateFarm(_) => invalid("duplicate_farm"),
            BriefError::RangeEndsBeforeItStarts | BriefError::RangeTooLong { .. } => {
                invalid("bad_range")
            }
            BriefError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Brief(#[from] BriefError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Brief(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_ways_a_range_can_be_wrong_share_the_code_the_dashboard_reads() {
        for error in [
            BriefError::RangeEndsBeforeItStarts,
            BriefError::RangeTooLong { max: 92 },
        ] {
            let info = error.to_error_info();

            assert_eq!(info.kind, ErrorKind::InvalidInput);
            assert_eq!(info.code, "bad_range");
        }
    }

    #[test]
    fn a_farm_named_twice_is_invalid_input_not_a_conflict() {
        let info = BriefError::DuplicateFarm(12).to_error_info();

        assert_eq!(
            info.kind,
            ErrorKind::InvalidInput,
            "the clash is inside one request body, not with stored data"
        );
        assert_eq!(info.code, "duplicate_farm");
        assert!(info.detail.contains("12"));
    }

    #[test]
    fn every_count_rule_has_a_code_the_job_can_act_on() {
        for (error, code) in [
            (BriefError::TooManyPoints { max: 8 }, "bad_points"),
            (BriefError::TooManySources { max: 12 }, "bad_sources"),
            (BriefError::FarmCount { min: 1, max: 2_000 }, "bad_farms"),
        ] {
            let info = error.to_error_info();

            assert_eq!(info.kind, ErrorKind::InvalidInput);
            assert_eq!(info.code, code);
        }
    }

    #[test]
    fn a_text_that_breaks_its_limit_keeps_the_default_code() {
        let info = BriefError::from(DomainError::InvalidValue("x".to_string())).to_error_info();

        assert_eq!(info.code, "invalid");
    }
}
