use crate::{features::alerts::domain::AlertError, shared::DomainError};

const MAX_LENGTH: usize = 120;

/// The name a job gives one of its alerts, unique within a farm, for example
/// `frost:2026-10-11` or `fire:firms-123`. A job that runs again sends the
/// same key and so replaces its alert instead of adding another.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AlertKey(String);

impl AlertKey {
    pub fn new(value: String) -> Result<Self, AlertError> {
        let length = value.chars().count();

        // No trimming: a key that differs by a space would be a second
        // alert, so such a key is refused instead of quietly changed.
        if length == 0
            || length > MAX_LENGTH
            || value
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(DomainError::InvalidValue(format!(
                "Alert key must be 1 to {MAX_LENGTH} characters with no spaces"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&AlertKey> for String {
    fn from(value: &AlertKey) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_type_and_a_day_or_an_external_id_make_a_key() {
        assert_eq!(
            AlertKey::new("frost:2026-10-11".to_string())
                .expect("key")
                .as_str(),
            "frost:2026-10-11"
        );
        assert!(AlertKey::new("fire:firms-VIIRS_1".to_string()).is_ok());
    }

    #[test]
    fn an_empty_key_or_one_with_a_space_is_refused() {
        assert!(AlertKey::new(String::new()).is_err());
        assert!(AlertKey::new("frost 2026".to_string()).is_err());
        assert!(AlertKey::new(" frost".to_string()).is_err());
        assert!(AlertKey::new("frost\n".to_string()).is_err());
    }

    #[test]
    fn the_limit_is_inclusive_and_counts_characters() {
        assert!(AlertKey::new("س".repeat(MAX_LENGTH)).is_ok());
        assert!(AlertKey::new("س".repeat(MAX_LENGTH + 1)).is_err());
    }
}
