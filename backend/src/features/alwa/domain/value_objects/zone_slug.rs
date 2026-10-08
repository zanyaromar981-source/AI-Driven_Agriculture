use crate::{features::alwa::domain::AlwaError, shared::DomainError};

use super::is_slug;

/// The zone a crop comes from, by the slug the zones slice gives it, for
/// example `chamchamal`. Only the spelling is checked here: the zones
/// themselves belong to another slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZoneSlug(String);

impl ZoneSlug {
    pub fn new(value: String) -> Result<Self, AlwaError> {
        if !is_slug(&value) {
            return Err(DomainError::InvalidValue(
                "Zone slug must be 1 to 40 lower-case letters and hyphens".to_string(),
            )
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
        assert!(ZoneSlug::new("a".repeat(40)).is_ok());
        assert!(ZoneSlug::new("a".repeat(41)).is_err());
        assert!(ZoneSlug::new(String::new()).is_err());
    }

    #[test]
    fn rejects_capitals_digits_and_spaces() {
        assert!(ZoneSlug::new("Chamchamal".to_string()).is_err());
        assert!(ZoneSlug::new("zone-1".to_string()).is_err());
        assert!(ZoneSlug::new("dashti hawler".to_string()).is_err());
        assert!(ZoneSlug::new("dashti_hawler".to_string()).is_err());
    }
}
