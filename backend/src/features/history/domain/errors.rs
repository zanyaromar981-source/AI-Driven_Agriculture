use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum HistoryError {
    #[error("The unit of {metric} is {expected}")]
    UnitMismatch {
        metric: String,
        expected: &'static str,
    },

    #[error("A push carries between {min} and {max} monthly points")]
    PointCount { min: usize, max: usize },

    #[error("The month {0} appears more than once in the push")]
    DuplicateMonth(String),

    #[error("The value {value} for {month} is outside {min} to {max}, the range of {metric}")]
    ValueOutOfRange {
        metric: String,
        month: String,
        value: f64,
        min: f64,
        max: f64,
    },

    #[error("A history window runs from an earlier month to a later one, {max} months at most")]
    BadWindow { max: u32 },

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
