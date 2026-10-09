use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::outlooks::domain::OutlookError,
    shared::DomainError,
};

impl ToErrorInfo for OutlookError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            OutlookError::InvalidSeason => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "bad_season", self.to_string())
            }
            OutlookError::InvalidIssueMonth => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "bad_issue_month", self.to_string())
            }
            OutlookError::RightOutOfRange { .. } => ErrorInfo::with_code(
                ErrorKind::InvalidInput,
                "bad_track_record",
                self.to_string(),
            ),
            OutlookError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Outlook(#[from] OutlookError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),

    #[error("No outlook has been issued yet")]
    NoOutlookIssued,

    #[error("No outlook has been issued for season {0}")]
    SeasonNotIssued(String),

    #[error("No outlook was issued for season {season} in {issued}")]
    IssueNotFound { season: String, issued: String },

    #[error("The zone {zone_slug} already has an outlook for season {season} issued in {issued}")]
    ZoneOutlookAlreadyExists {
        zone_slug: String,
        season: String,
        issued: String,
    },

    #[error("The zone {zone_slug} has no outlook for season {season} issued in {issued}")]
    ZoneOutlookNotFound {
        zone_slug: String,
        season: String,
        issued: String,
    },

    #[error("Season {season} already has a track record for the issue of {issued}")]
    RunAlreadyExists { season: String, issued: String },

    #[error("Season {season} has no track record for the issue of {issued}")]
    RunNotFound { season: String, issued: String },
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Outlook(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
            AppError::NoOutlookIssued
            | AppError::SeasonNotIssued(_)
            | AppError::IssueNotFound { .. }
            | AppError::ZoneOutlookNotFound { .. }
            | AppError::RunNotFound { .. } => ErrorInfo::new(ErrorKind::NotFound, self.to_string()),
            AppError::ZoneOutlookAlreadyExists { .. } | AppError::RunAlreadyExists { .. } => {
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
        let info = OutlookError::InvalidSeason.to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "bad_season");
    }

    #[test]
    fn an_impossible_track_record_is_invalid_input() {
        let info = OutlookError::RightOutOfRange {
            tested: 25,
            right: 26,
        }
        .to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "bad_track_record");
    }

    #[test]
    fn an_outlook_that_was_never_issued_is_not_found_rather_than_made_up() {
        for error in [
            AppError::NoOutlookIssued,
            AppError::SeasonNotIssued("2026-27".to_string()),
            AppError::IssueNotFound {
                season: "2026-27".to_string(),
                issued: "2026-10".to_string(),
            },
        ] {
            let info = error.to_error_info();

            assert_eq!(info.kind, ErrorKind::NotFound);
            assert_eq!(info.code, "not_found");
        }
    }
}
