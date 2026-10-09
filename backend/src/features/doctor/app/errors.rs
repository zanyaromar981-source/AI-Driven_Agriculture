use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::doctor::domain::DoctorError,
    shared::DomainError,
};

impl ToErrorInfo for DoctorError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            DoctorError::EmptyQuestion => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "empty_question", self.to_string())
            }
            DoctorError::TooManyPhotos(_)
            | DoctorError::PhotoTooLarge(_)
            | DoctorError::PhotoNotAnImage => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "bad_photo", self.to_string())
            }
            DoctorError::QuestionTooLong(_) => {
                ErrorInfo::new(ErrorKind::InvalidInput, self.to_string())
            }
            DoctorError::BadAnswer(_) => ErrorInfo::with_code(
                ErrorKind::UpstreamInvalidResponse,
                "doctor_failed",
                self.to_string(),
            ),
            DoctorError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Doctor(#[from] DoctorError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),

    /// The Doctor service is up but said it cannot answer yet (it has no AI
    /// key). The app may try again later.
    #[error("The Doctor service is not ready to answer")]
    DoctorNotReady,

    /// The Doctor service is down, timed out, failed or answered something
    /// that cannot be used. The cause is logged where it happened.
    #[error("The Doctor service failed to answer")]
    DoctorFailed,
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Doctor(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
            AppError::DoctorNotReady => ErrorInfo::with_code(
                ErrorKind::UpstreamUnavailable,
                "doctor_not_ready",
                self.to_string(),
            ),
            AppError::DoctorFailed => ErrorInfo::with_code(
                ErrorKind::UpstreamFailure,
                "doctor_failed",
                self.to_string(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asking_nothing_is_an_empty_question_the_app_can_read() {
        let info = DoctorError::EmptyQuestion.to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "empty_question");
    }

    #[test]
    fn every_photo_problem_is_one_bad_photo_code() {
        for error in [
            DoctorError::TooManyPhotos(6),
            DoctorError::PhotoTooLarge(4),
            DoctorError::PhotoNotAnImage,
        ] {
            let info = error.to_error_info();

            assert_eq!(info.kind, ErrorKind::InvalidInput, "{error:?}");
            assert_eq!(info.code, "bad_photo", "{error:?}");
        }
    }

    #[test]
    fn a_question_too_long_is_invalid_input() {
        let info = DoctorError::QuestionTooLong(1_000).to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "invalid");
    }

    #[test]
    fn a_doctor_not_ready_is_told_apart_from_a_doctor_that_failed() {
        let not_ready = AppError::DoctorNotReady.to_error_info();
        let failed = AppError::DoctorFailed.to_error_info();

        assert_eq!(not_ready.kind, ErrorKind::UpstreamUnavailable);
        assert_eq!(not_ready.code, "doctor_not_ready");
        assert_eq!(failed.kind, ErrorKind::UpstreamFailure);
        assert_eq!(failed.code, "doctor_failed");
    }
}
