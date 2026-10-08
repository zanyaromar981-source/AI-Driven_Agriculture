use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum OutlookError {
    #[error(
        "A season looks like 2026-27: a year, a hyphen and the last two digits of the next year"
    )]
    InvalidSeason,

    #[error("An issue month looks like 2026-10")]
    InvalidIssueMonth,

    #[error("The method cannot have been right in {right} of {tested} seasons")]
    RightOutOfRange { tested: i32, right: i32 },

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
