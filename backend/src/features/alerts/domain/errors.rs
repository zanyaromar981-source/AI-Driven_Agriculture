use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum AlertError {
    #[error(transparent)]
    DomainError(#[from] DomainError),
}
