use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum InsightError {
    #[error("A reading needs between {min} and {max} measures")]
    MeasureCount { min: usize, max: usize },

    #[error("The measure code {0} appears more than once in the reading")]
    DuplicateMeasureCode(String),

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
