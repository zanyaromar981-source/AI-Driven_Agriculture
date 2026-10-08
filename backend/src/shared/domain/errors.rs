#[derive(thiserror::Error, Debug)]
pub enum DomainError {
    #[error("Invalid value: {0}")]
    InvalidValue(String),

    #[error("Duplicate value: {0}")]
    DuplicateValue(String),
}
