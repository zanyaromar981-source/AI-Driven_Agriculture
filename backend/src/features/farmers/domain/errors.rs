use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum FarmerError {
    #[error("The code is wrong")]
    WrongCode,

    #[error("The code has expired")]
    CodeExpired,

    #[error("A code was sent a moment ago, wait {0} seconds")]
    CodeRequestedTooSoon(u64),

    #[error("This account is blocked")]
    Blocked,

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
