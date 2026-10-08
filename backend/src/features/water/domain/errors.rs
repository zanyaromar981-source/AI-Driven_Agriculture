use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum WaterError {
    #[error(
        "A season looks like 2026-27: a year, a hyphen and the last two digits of the next year"
    )]
    InvalidSeason,

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
