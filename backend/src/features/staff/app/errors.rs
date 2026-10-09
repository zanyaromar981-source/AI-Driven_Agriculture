use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::staff::domain::StaffError,
    shared::DomainError,
};

impl ToErrorInfo for StaffError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            StaffError::SystemRole => {
                ErrorInfo::with_code(ErrorKind::Conflict, "system_role", self.to_string())
            }
            StaffError::RoleInUse => {
                ErrorInfo::with_code(ErrorKind::Conflict, "role_in_use", self.to_string())
            }
            StaffError::RoleNameTaken => {
                ErrorInfo::with_code(ErrorKind::Conflict, "role_name_taken", self.to_string())
            }
            StaffError::EmailTaken => {
                ErrorInfo::with_code(ErrorKind::Conflict, "email_taken", self.to_string())
            }
            // One code for a wrong email, a wrong password and an account
            // that is switched off: the answer must not say which it was.
            StaffError::BadCredentials => ErrorInfo::with_code(
                ErrorKind::Authentication,
                "bad_credentials",
                self.to_string(),
            ),
            StaffError::OwnAccount => {
                ErrorInfo::with_code(ErrorKind::Conflict, "own_account", self.to_string())
            }
            StaffError::LastOwner => {
                ErrorInfo::with_code(ErrorKind::Conflict, "last_owner", self.to_string())
            }
            StaffError::UnknownRole => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "unknown_role", self.to_string())
            }
            StaffError::CannotGrant => {
                ErrorInfo::with_code(ErrorKind::Authorization, "cannot_grant", self.to_string())
            }
            // Only someone already signed in can get this answer, about
            // their own account, so it may say what was wrong.
            StaffError::WrongPassword => {
                ErrorInfo::with_code(ErrorKind::Authorization, "wrong_password", self.to_string())
            }
            StaffError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Staff(#[from] StaffError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Staff(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_broken_rule_is_a_conflict_with_its_own_code() {
        for (error, code) in [
            (StaffError::SystemRole, "system_role"),
            (StaffError::RoleInUse, "role_in_use"),
            (StaffError::RoleNameTaken, "role_name_taken"),
            (StaffError::EmailTaken, "email_taken"),
            (StaffError::OwnAccount, "own_account"),
            (StaffError::LastOwner, "last_owner"),
        ] {
            let info = error.to_error_info();

            assert_eq!(info.kind, ErrorKind::Conflict, "{code}");
            assert_eq!(info.code, code);
        }
    }

    #[test]
    fn a_failed_sign_in_is_an_authentication_failure_with_one_code() {
        let info = StaffError::BadCredentials.to_error_info();

        assert_eq!(info.kind, ErrorKind::Authentication);
        assert_eq!(info.code, "bad_credentials");
    }

    #[test]
    fn granting_beyond_ones_own_permissions_is_forbidden_with_its_own_code() {
        let info = StaffError::CannotGrant.to_error_info();

        assert_eq!(info.kind, ErrorKind::Authorization);
        assert_eq!(info.code, "cannot_grant");
    }

    #[test]
    fn a_wrong_current_password_is_forbidden_with_its_own_code() {
        let info = StaffError::WrongPassword.to_error_info();

        assert_eq!(info.kind, ErrorKind::Authorization);
        assert_eq!(info.code, "wrong_password");
    }

    #[test]
    fn assigning_a_role_that_does_not_exist_is_invalid_input() {
        let info = StaffError::UnknownRole.to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "unknown_role");
    }
}
