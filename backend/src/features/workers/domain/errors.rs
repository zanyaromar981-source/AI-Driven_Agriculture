use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum WorkerError {
    #[error("Name must be 1 to {0} characters")]
    BadName(usize),

    #[error("Cost must be {min} to {max} dinars")]
    BadCost { min: i64, max: i64 },

    #[error("Note must be {0} characters max")]
    NoteTooLong(usize),

    #[error("Cost is per day or per hour")]
    UnknownCostPer,

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
