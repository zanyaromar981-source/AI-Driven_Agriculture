use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::dams::domain::DamError,
    shared::DomainError,
};

impl ToErrorInfo for DamError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            DamError::VolumeOverCapacity { .. } => ErrorInfo::with_code(
                ErrorKind::InvalidInput,
                "volume_over_capacity",
                self.to_string(),
            ),
            DamError::RangeEndsBeforeItStarts => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "bad_range", self.to_string())
            }
            DamError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Dam(#[from] DamError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),

    #[error("There is no dam called {0}")]
    DamNotFound(String),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Dam(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
            AppError::DamNotFound(_) => ErrorInfo::new(ErrorKind::NotFound, self.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_volume_over_capacity_has_its_own_code_for_the_data_job() {
        let info = DamError::VolumeOverCapacity {
            volume_bn_m3: 9.0,
            capacity_bn_m3: 6.97,
        }
        .to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "volume_over_capacity");
    }

    #[test]
    fn an_unknown_dam_is_not_found_rather_than_invalid() {
        let info = AppError::DamNotFound("mosul".to_string()).to_error_info();

        assert_eq!(info.kind, ErrorKind::NotFound);
        assert_eq!(info.code, "not_found");
    }
}
