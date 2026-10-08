use crate::{features::farmers::domain::FarmerError, shared::DomainError};

const DIGITS: usize = 6;

/// The six digit code sent to a phone to prove the farmer holds it.
#[derive(Clone, PartialEq, Eq)]
pub struct SignInCode(String);

impl SignInCode {
    pub fn new(value: String) -> Result<Self, FarmerError> {
        if value.len() != DIGITS || !value.chars().all(|character| character.is_ascii_digit()) {
            return Err(
                DomainError::InvalidValue(format!("The code must be {DIGITS} digits")).into(),
            );
        }

        Ok(Self(value))
    }

    /// Builds the code from a number, keeping leading zeros.
    pub fn from_number(number: u32) -> Self {
        Self(format!("{:0width$}", number % 1_000_000, width = DIGITS))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The code is a secret for a few minutes, so it stays out of debug output.
impl std::fmt::Debug for SignInCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SignInCode(******)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_six_digits() {
        assert!(SignInCode::new("123456".to_string()).is_ok());
        assert!(SignInCode::new("000000".to_string()).is_ok());
    }

    #[test]
    fn rejects_anything_else() {
        assert!(SignInCode::new("12345".to_string()).is_err());
        assert!(SignInCode::new("1234567".to_string()).is_err());
        assert!(SignInCode::new("12a456".to_string()).is_err());
        assert!(SignInCode::new("١٢٣٤٥٦".to_string()).is_err());
    }

    #[test]
    fn a_small_number_keeps_its_leading_zeros() {
        assert_eq!(SignInCode::from_number(42).as_str(), "000042");
        assert_eq!(SignInCode::from_number(1_000_042).as_str(), "000042");
    }

    #[test]
    fn debug_output_never_shows_the_code() {
        let shown = format!("{:?}", SignInCode::from_number(123_456));

        assert!(!shown.contains("123456"));
    }
}
