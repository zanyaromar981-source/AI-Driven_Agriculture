use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::messages::domain::MessageError,
    shared::DomainError,
};

impl ToErrorInfo for MessageError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            MessageError::TextTooLong(_) => {
                ErrorInfo::new(ErrorKind::InvalidInput, self.to_string())
            }
            MessageError::TooManyPhotos(_)
            | MessageError::PhotoTooLarge(_)
            | MessageError::PhotoNotAnImage => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "bad_photo", self.to_string())
            }
            MessageError::TooManyMessages { retry_after_s, .. } => {
                ErrorInfo::new(ErrorKind::RateLimited, self.to_string()).retry_after(*retry_after_s)
            }
            MessageError::NoReply => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "no_reply", self.to_string())
            }
            MessageError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Message(#[from] MessageError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Message(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_photo_problem_is_one_bad_photo_code() {
        for error in [
            MessageError::TooManyPhotos(4),
            MessageError::PhotoTooLarge(4),
            MessageError::PhotoNotAnImage,
        ] {
            let info = error.to_error_info();

            assert_eq!(info.kind, ErrorKind::InvalidInput, "{error:?}");
            assert_eq!(info.code, "bad_photo", "{error:?}");
        }
    }

    #[test]
    fn too_many_messages_is_a_rate_limit_that_says_how_long_to_wait() {
        let info = MessageError::TooManyMessages {
            max: 20,
            retry_after_s: 90,
        }
        .to_error_info();

        assert_eq!(info.kind, ErrorKind::RateLimited);
        assert_eq!(info.code, "rate_limited");
        assert_eq!(info.retry_after_s, Some(90));
    }

    #[test]
    fn marking_an_unanswered_message_as_replied_has_its_own_code() {
        let info = MessageError::NoReply.to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "no_reply");
    }

    #[test]
    fn text_too_long_is_invalid_input() {
        let info = MessageError::TextTooLong(2_000).to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "invalid");
    }
}
