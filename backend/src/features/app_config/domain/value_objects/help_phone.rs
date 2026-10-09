use crate::{features::app_config::domain::AppConfigError, shared::DomainError};

const MIN_DIGITS: usize = 3;
const MAX_DIGITS: usize = 15;

/// The number the app tells a farmer to call for help. It may be a short
/// code or a landline, so it is not held to the rules of a farmer's mobile
/// number: digits, with an optional `+` in front.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HelpPhone(String);

impl HelpPhone {
    pub fn new(value: String) -> Result<Self, AppConfigError> {
        // Spaces and hyphens are how people write numbers; they are dropped.
        let value: String = value
            .chars()
            .filter(|character| !character.is_whitespace() && *character != '-')
            .collect();

        let digits = value.strip_prefix('+').unwrap_or(&value);

        if !(MIN_DIGITS..=MAX_DIGITS).contains(&digits.len())
            || !digits.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(DomainError::InvalidValue(format!(
                "Help phone must be {MIN_DIGITS} to {MAX_DIGITS} digits, with an optional + in front"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&HelpPhone> for String {
    fn from(value: &HelpPhone) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_mobile_number_a_landline_and_a_short_code() {
        for number in ["+9647501234567", "0533301234", "115"] {
            assert!(HelpPhone::new(number.to_string()).is_ok(), "{number}");
        }
    }

    #[test]
    fn spaces_and_hyphens_are_dropped() {
        let phone = HelpPhone::new("+964 750 123-4567".to_string()).expect("phone");

        assert_eq!(phone.as_str(), "+9647501234567");
    }

    #[test]
    fn letters_a_misplaced_plus_and_wrong_lengths_are_refused() {
        for number in ["", "12", "call us", "964+750", "1234567890123456", "+"] {
            assert!(HelpPhone::new(number.to_string()).is_err(), "{number:?}");
        }
    }
}
