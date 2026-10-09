use thiserror::Error;

use crate::app::{AppError as GlobalAppError, ErrorInfo, ToErrorInfo};

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::GlobalAppError(err) => err.to_error_info(),
        }
    }
}
