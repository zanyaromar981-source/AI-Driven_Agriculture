use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum FarmError {
    #[error("A farm outline needs between {min} and {max} corners")]
    OutlineCornerCount { min: usize, max: usize },

    #[error("The farm outline crosses itself")]
    OutlineSelfIntersects,

    #[error("The farm outline encloses no cells")]
    OutlineEnclosesNoCells,

    #[error("The farm is larger than the allowed {0} cells")]
    TooManyCells(usize),

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
