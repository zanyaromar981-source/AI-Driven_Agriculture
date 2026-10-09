use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::zones::domain::ZoneError,
    shared::DomainError,
};

impl ToErrorInfo for ZoneError {
    fn to_error_info(&self) -> ErrorInfo {
        let invalid = |code| ErrorInfo::with_code(ErrorKind::InvalidInput, code, self.to_string());

        match self {
            // A data job fixes a month the same way wherever it was wrong.
            ZoneError::BadMonth { .. } | ZoneError::BadCalendarMonth => invalid("bad_month"),
            ZoneError::FromAfterTo => invalid("bad_range"),
            ZoneError::BadYear { .. } => invalid("bad_year"),
            ZoneError::SameYear => invalid("same_year"),
            ZoneError::DrynessOutOfRange => invalid("bad_dryness"),
            ZoneError::RainOutOfRange => invalid("bad_rain_pct_of_normal"),
            ZoneError::GreennessOutOfRange => invalid("bad_greenness_pct_vs_normal"),
            ZoneError::WaterNeedOutOfRange => invalid("bad_water_need"),
            ZoneError::UnknownCrop(_) | ZoneError::RepeatedCrop(_) => invalid("bad_crop"),
            ZoneError::BadShape => invalid("bad_shape"),
            ZoneError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Zone(#[from] ZoneError),

    #[error("A reading for this month is already stored: update it instead")]
    ReadingAlreadyExists,

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Zone(err) => err.to_error_info(),
            AppError::ReadingAlreadyExists => {
                ErrorInfo::with_code(ErrorKind::Conflict, "already_exists", self.to_string())
            }
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_month_wrong_in_a_path_or_in_a_query_carries_the_same_code() {
        for error in [
            ZoneError::BadMonth {
                min_year: 2000,
                max_year: 2100,
            },
            ZoneError::BadCalendarMonth,
        ] {
            let info = error.to_error_info();

            assert_eq!(info.kind, ErrorKind::InvalidInput);
            assert_eq!(info.code, "bad_month");
        }
    }

    #[test]
    fn each_measure_out_of_range_names_itself() {
        for (error, code) in [
            (ZoneError::DrynessOutOfRange, "bad_dryness"),
            (ZoneError::RainOutOfRange, "bad_rain_pct_of_normal"),
            (
                ZoneError::GreennessOutOfRange,
                "bad_greenness_pct_vs_normal",
            ),
            (ZoneError::WaterNeedOutOfRange, "bad_water_need"),
            (ZoneError::UnknownCrop("rice".to_string()), "bad_crop"),
            (ZoneError::RepeatedCrop("wheat".to_string()), "bad_crop"),
            (ZoneError::SameYear, "same_year"),
            (ZoneError::FromAfterTo, "bad_range"),
            (ZoneError::BadShape, "bad_shape"),
            (
                ZoneError::BadYear {
                    min: 2000,
                    max: 2100,
                },
                "bad_year",
            ),
        ] {
            let info = error.to_error_info();

            assert_eq!(info.kind, ErrorKind::InvalidInput, "{code}");
            assert_eq!(info.code, code);
        }
    }

    #[test]
    fn creating_a_reading_that_is_already_stored_is_a_conflict_a_client_can_tell_apart() {
        let info = AppError::ReadingAlreadyExists.to_error_info();

        assert_eq!(info.kind, ErrorKind::Conflict);
        assert_eq!(info.code, "already_exists");
    }

    #[test]
    fn an_unknown_zone_is_not_found() {
        let info = AppError::from(GlobalAppError::NotFound).to_error_info();

        assert_eq!(info.kind, ErrorKind::NotFound);
        assert_eq!(info.code, "not_found");
    }
}
