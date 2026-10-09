use crate::{features::workers::domain::WorkerError, shared::DomainError};

const MAX_LENGTH: usize = 40;

/// The district a worker is in, by the slug the zones slice gives it, for
/// example `chamchamal`. Only the spelling is checked here: the zones
/// themselves belong to another slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZoneSlug(String);

impl ZoneSlug {
    pub fn new(value: String) -> Result<Self, WorkerError> {
        let is_slug = !value.is_empty()
            && value.len() <= MAX_LENGTH
            && value
                .chars()
                .all(|character| character.is_ascii_lowercase() || character == '-');

        if !is_slug {
            return Err(DomainError::InvalidValue(format!(
                "Zone slug must be 1 to {MAX_LENGTH} lower-case letters and hyphens"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&ZoneSlug> for String {
    fn from(value: &ZoneSlug) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_lower_case_letters_and_hyphens() {
        assert!(ZoneSlug::new("chamchamal".to_string()).is_ok());
        assert!(ZoneSlug::new("dashti-hawler".to_string()).is_ok());
    }

    #[test]
    fn the_length_limit_is_inclusive() {
        assert!(ZoneSlug::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(ZoneSlug::new("a".repeat(MAX_LENGTH + 1)).is_err());
        assert!(ZoneSlug::new(String::new()).is_err());
    }

    #[test]
    fn rejects_capitals_digits_and_spaces() {
        assert!(ZoneSlug::new("Chamchamal".to_string()).is_err());
        assert!(ZoneSlug::new("zone-1".to_string()).is_err());
        assert!(ZoneSlug::new("dashti hawler".to_string()).is_err());
    }
}
