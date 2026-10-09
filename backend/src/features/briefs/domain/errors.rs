use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum BriefError {
    #[error("A brief carries at most {max} points")]
    TooManyPoints { max: usize },

    #[error("A brief names at most {max} sources")]
    TooManySources { max: usize },

    #[error("A push names between {min} and {max} farms")]
    FarmCount { min: usize, max: usize },

    #[error("The farm {0} appears more than once in the push")]
    DuplicateFarm(i32),

    #[error("The range must not end before it starts")]
    RangeEndsBeforeItStarts,

    #[error("The range must not be longer than {max} days")]
    RangeTooLong { max: i64 },

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
