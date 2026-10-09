use crate::{features::staff::domain::StaffError, shared::DomainError};

const MIN_LENGTH: usize = 10;
const MAX_LENGTH: usize = 200;

/// A password as typed. It lives only for the length of a request: what is
/// stored is its hash.
#[derive(Clone, PartialEq, Eq)]
pub struct Password(String);

impl Password {
    /// A password being set for an account. It is taken exactly as typed:
    /// spaces count.
    pub fn new(value: String) -> Result<Self, StaffError> {
        let length = value.chars().count();

        if length < MIN_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Password must be at least {MIN_LENGTH} characters"
            ))
            .into());
        }

        if length > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Password must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    /// A password presented at sign-in. It is not held to the rules for a
    /// new one, so the answer never says how a password must look. One that
    /// is longer than any stored password can be is refused before hashing.
    pub fn presented(value: String) -> Result<Self, StaffError> {
        if value.chars().count() > MAX_LENGTH {
            return Err(StaffError::BadCredentials);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A password must never reach a log, so it stays out of debug output.
impl std::fmt::Debug for Password {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Password(******)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_password_needs_ten_to_two_hundred_characters() {
        assert!(Password::new("a".repeat(MIN_LENGTH - 1)).is_err());
        assert!(Password::new("a".repeat(MIN_LENGTH)).is_ok());
        assert!(Password::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(Password::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn the_limits_count_characters_not_bytes() {
        assert!(Password::new("ک".repeat(MIN_LENGTH)).is_ok());
        assert!(Password::new("ک".repeat(MAX_LENGTH)).is_ok());
    }

    #[test]
    fn a_password_is_kept_exactly_as_typed() {
        let password = Password::new("  spaces count  ".to_string()).expect("password");

        assert_eq!(password.as_str(), "  spaces count  ");
    }

    #[test]
    fn a_short_password_may_still_be_presented_at_sign_in() {
        assert!(Password::presented("short".to_string()).is_ok());
    }

    #[test]
    fn an_over_long_presented_password_is_just_bad_credentials() {
        assert!(matches!(
            Password::presented("a".repeat(MAX_LENGTH + 1)),
            Err(StaffError::BadCredentials)
        ));
    }

    #[test]
    fn debug_output_never_shows_the_password() {
        let shown = format!(
            "{:?}",
            Password::new("correct horse battery".to_string()).expect("password")
        );

        assert!(!shown.contains("horse"));
    }
}
