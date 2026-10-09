use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum AppConfigError {
    #[error("A version must be three numbers like 1.10.0, got {0:?}")]
    BadVersion(String),

    #[error("min_version ({min}) may not be above latest_version ({latest})")]
    MinAboveLatest { min: String, latest: String },

    #[error("{0}")]
    BadLimits(String),

    #[error("maintenance_from must be before maintenance_until")]
    BadMaintenanceWindow,

    #[error("{0} is needed while its switch is on")]
    MissingText(&'static str),

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
