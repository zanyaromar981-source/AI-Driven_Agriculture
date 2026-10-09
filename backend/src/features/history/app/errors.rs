use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::history::domain::HistoryError,
    shared::DomainError,
};

impl ToErrorInfo for HistoryError {
    fn to_error_info(&self) -> ErrorInfo {
        let code = match self {
            HistoryError::UnitMismatch { .. } => "wrong_unit",
            HistoryError::PointCount { .. } => "bad_points",
            HistoryError::DuplicateMonth(_) => "duplicate_month",
            HistoryError::ValueOutOfRange { .. } => "value_out_of_range",
            HistoryError::BadWindow { .. } => "bad_window",
            HistoryError::DomainError(err) => return err.to_error_info(),
        };

        ErrorInfo::with_code(ErrorKind::InvalidInput, code, self.to_string())
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    History(#[from] HistoryError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::History(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_rule_a_job_can_break_has_its_own_code() {
        let cases = [
            (
                HistoryError::UnitMismatch {
                    metric: "rain_mm".to_string(),
                    expected: "mm",
                },
                "wrong_unit",
            ),
            (HistoryError::PointCount { min: 1, max: 240 }, "bad_points"),
            (
                HistoryError::DuplicateMonth("2020-01".to_string()),
                "duplicate_month",
            ),
            (
                HistoryError::ValueOutOfRange {
                    metric: "rain_mm".to_string(),
                    month: "2020-01".to_string(),
                    value: -1.0,
                    min: 0.0,
                    max: 2_000.0,
                },
                "value_out_of_range",
            ),
            (HistoryError::BadWindow { max: 120 }, "bad_window"),
        ];

        for (error, code) in cases {
            let info = error.to_error_info();

            assert_eq!(info.kind, ErrorKind::InvalidInput, "{code}");
            assert_eq!(info.code, code);
        }
    }

    #[test]
    fn a_repeated_month_names_the_month_and_is_not_a_conflict() {
        let info = HistoryError::DuplicateMonth("2020-01".to_string()).to_error_info();

        assert_eq!(
            info.kind,
            ErrorKind::InvalidInput,
            "the clash is inside one request body, not with stored data"
        );
        assert!(info.detail.contains("2020-01"));
    }

    #[test]
    fn the_wrong_unit_tells_the_job_the_right_one() {
        let info = HistoryError::UnitMismatch {
            metric: "rain_mm".to_string(),
            expected: "mm",
        }
        .to_error_info();

        assert!(info.detail.contains("mm"));
    }
}
