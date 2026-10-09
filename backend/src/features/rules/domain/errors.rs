use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum RuleError {
    #[error("{code} must be between {min} and {max}, and {value} is not")]
    ValueOutOfRange {
        code: String,
        value: f64,
        min: f64,
        max: f64,
    },

    #[error("The value must be a number")]
    ValueNotANumber,

    #[error("The reason must be {min} to {max} characters long")]
    ReasonLength { min: usize, max: usize },

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
