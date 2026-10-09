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

    #[error("The plan for season {season} already has an entry for the zone {zone_slug}")]
    EntryAlreadyExists { season: String, zone_slug: String },

    #[error("The plan for season {season} has no entry for the zone {zone_slug}")]
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
            AppError::EntryAlreadyExists { .. } => {
                ErrorInfo::with_code(ErrorKind::Conflict, "already_exists", self.to_string())
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
    fn a_missing_plan_is_not_found() {
        let info = AppError::NoWaterPlan.to_error_info();

        assert_eq!(info.kind, ErrorKind::NotFound);
        assert_eq!(info.code, "not_found");
    }

    #[test]
    fn a_second_entry_for_the_same_zone_is_a_conflict_the_dashboard_can_name() {
        let info = AppError::EntryAlreadyExists {
            season: "2026-27".to_string(),
            zone_slug: "makhmur".to_string(),
        }
        .to_error_info();

        assert_eq!(info.kind, ErrorKind::Conflict);
        assert_eq!(info.code, "already_exists");
    }

    #[test]
    fn a_missing_entry_is_not_found() {
        let info = AppError::EntryNotFound {
            season: "2026-27".to_string(),
            zone_slug: "makhmur".to_string(),
        }
        .to_error_info();

        assert_eq!(info.kind, ErrorKind::NotFound);
        assert_eq!(info.code, "not_found");
    }
}
