use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::water::domain::WaterError,
    shared::DomainError,
};

impl ToErrorInfo for WaterError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            WaterError::InvalidSeason => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "bad_season", self.to_string())
            }
            WaterError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Water(#[from] WaterError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),

    #[error("No water plan has been made yet")]
    NoWaterPlan,

    #[error("The water plan for season {season} has no entry for zone {zone_slug}")]
    EntryNotFound { season: String, zone_slug: String },
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Water(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
            AppError::NoWaterPlan | AppError::EntryNotFound { .. } => {
                ErrorInfo::new(ErrorKind::NotFound, self.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_malformed_season_has_its_own_code_for_the_data_job() {
        let info = WaterError::InvalidSeason.to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "bad_season");
    }

    #[test]
    fn a_missing_plan_or_entry_is_not_found() {
        for error in [
            AppError::NoWaterPlan,
            AppError::EntryNotFound {
                season: "2026-27".to_string(),
                zone_slug: "makhmur".to_string(),
            },
        ] {
            let info = error.to_error_info();

            assert_eq!(info.kind, ErrorKind::NotFound);
            assert_eq!(info.code, "not_found");
        }
    }
}
