use crate::shared::domain::DomainError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const COUNTRY_CODE: &str = "+964";
const NATIONAL_DIGITS: usize = 10;

/// An Iraqi mobile number in E.164 form, for example `+9647501234567`. The
/// phone is the account: there is no name and no password.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct Phone(String);

impl Phone {
    pub fn new(value: String) -> Result<Self, DomainError> {
        let Some(national) = value.strip_prefix(COUNTRY_CODE) else {
            return Err(DomainError::InvalidValue(format!(
                "Phone must start with {COUNTRY_CODE}"
            )));
        };

        if national.len() != NATIONAL_DIGITS
            || !national.chars().all(|character| character.is_ascii_digit())
        {
            return Err(DomainError::InvalidValue(format!(
                "Phone must have {NATIONAL_DIGITS} digits after {COUNTRY_CODE}"
            )));
        }

        if !national.starts_with('7') {
            return Err(DomainError::InvalidValue(
                "Phone must be a mobile number".to_string(),
            ));
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<Phone> for String {
    fn from(value: Phone) -> Self {
        Self::from(&value)
    }
}

impl From<&Phone> for String {
    fn from(value: &Phone) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_an_iraqi_mobile_number() {
        assert!(Phone::new("+9647501234567".to_string()).is_ok());
    }

    #[test]
    fn rejects_a_number_without_the_country_code() {
        assert!(Phone::new("07501234567".to_string()).is_err());
        assert!(Phone::new("9647501234567".to_string()).is_err());
    }

    #[test]
    fn rejects_spaces_and_other_separators() {
        assert!(
            Phone::new("+964 750 123 4567".to_string()).is_err(),
            "the stored form has no spaces, so two spellings cannot be two accounts"
        );
        assert!(Phone::new("+964750-123456".to_string()).is_err());
    }

    #[test]
    fn rejects_the_wrong_number_of_digits() {
        assert!(Phone::new("+964750123456".to_string()).is_err());
        assert!(Phone::new("+96475012345678".to_string()).is_err());
    }

    #[test]
    fn rejects_a_landline() {
        assert!(Phone::new("+9645301234567".to_string()).is_err());
    }
}
