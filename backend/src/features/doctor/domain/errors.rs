use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum DoctorError {
    #[error("Ask a question or add at least one photo")]
    EmptyQuestion,

    #[error("A question must be {0} characters max")]
    QuestionTooLong(usize),

    #[error("At most {0} photos can be sent with one question")]
    TooManyPhotos(usize),

    #[error("A photo must be {0} MB max")]
    PhotoTooLarge(usize),

    #[error("A photo must be a JPEG or PNG image")]
    PhotoNotAnImage,

    #[error("The Doctor's answer cannot be used: {0}")]
    BadAnswer(String),

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
