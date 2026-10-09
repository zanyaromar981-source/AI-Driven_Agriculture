use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum FireError {
    #[error("The fire is outside the region: latitude 28 to 40, longitude 38 to 50")]
    OutsideRegion,

    #[error("The window must be between {min} and {max} hours")]
    WindowOutOfRange { min: i64, max: i64 },

    #[error("A fire with this external id is already stored")]
    AlreadyExists,

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
