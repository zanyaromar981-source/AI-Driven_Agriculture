use chrono::NaiveDate;
use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::dams::domain::DamError,
    shared::DomainError,
};

impl ToErrorInfo for DamError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            DamError::VolumeOverCapacity { .. } => ErrorInfo::with_code(
                ErrorKind::InvalidInput,
                "volume_over_capacity",
                self.to_string(),
            ),
            DamError::RangeEndsBeforeItStarts => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "bad_range", self.to_string())
            }
            DamError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Dam(#[from] DamError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),

    #[error("There is no dam called {0}")]
    DamNotFound(String),

    #[error("The dam {slug} already has a reading for {day}")]
    ReadingAlreadyExists { slug: String, day: NaiveDate },

    #[error("The dam {slug} has no reading for {day}")]
    ReadingNotFound { slug: String, day: NaiveDate },
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Dam(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
            AppError::DamNotFound(_) | AppError::ReadingNotFound { .. } => {
                ErrorInfo::new(ErrorKind::NotFound, self.to_string())
            }
            AppError::ReadingAlreadyExists { .. } => {
                ErrorInfo::with_code(ErrorKind::Conflict, "already_exists", self.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_volume_over_capacity_has_its_own_code_for_the_data_job() {
        let info = DamError::VolumeOverCapacity {
            volume_bn_m3: 9.0,
            capacity_bn_m3: 6.97,
        }
        .to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "volume_over_capacity");
    }

    #[test]
    fn an_unknown_dam_is_not_found_rather_than_invalid() {
        let info = AppError::DamNotFound("mosul".to_string()).to_error_info();

        assert_eq!(info.kind, ErrorKind::NotFound);
        assert_eq!(info.code, "not_found");
    }

    #[test]
    fn a_second_reading_for_the_same_day_is_a_conflict_the_dashboard_can_name() {
        let info = AppError::ReadingAlreadyExists {
            slug: "dukan".to_string(),
            day: NaiveDate::from_ymd_opt(2026, 10, 1).expect("day"),
        }
        .to_error_info();

        assert_eq!(info.kind, ErrorKind::Conflict);
        assert_eq!(info.code, "already_exists");
    }

    #[test]
    fn a_missing_reading_is_not_found() {
        let info = AppError::ReadingNotFound {
            slug: "dukan".to_string(),
            day: NaiveDate::from_ymd_opt(2026, 10, 1).expect("day"),
        }
        .to_error_info();

        assert_eq!(info.kind, ErrorKind::NotFound);
        assert_eq!(info.code, "not_found");
    }
}
