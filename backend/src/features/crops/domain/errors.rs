use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum CropError {
    #[error("`{0}` is the word for an unplanted cell, not a crop")]
    ReservedCode(String),

    #[error("A crop with this code already exists")]
    AlreadyExists,

    #[error("The crop is in use by a farm, a listing or a price; switch it off instead")]
    InUse,

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
