use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum MessageError {
    #[error("A message must be {0} characters max")]
    TextTooLong(usize),

    #[error("At most {0} photos can be sent with one message")]
    TooManyPhotos(usize),

    #[error("A photo must be {0} MB max")]
    PhotoTooLarge(usize),

    #[error("A photo must be a JPEG or PNG image")]
    PhotoNotAnImage,

    #[error("At most {max} messages can be sent in 24 hours")]
    TooManyMessages { max: u64, retry_after_s: u64 },

    #[error("A message with no reply cannot be marked as replied")]
    NoReply,

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
