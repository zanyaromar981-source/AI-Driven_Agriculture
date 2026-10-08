use crate::{features::fires::domain::FireError, shared::DomainError};

const MAX_LENGTH: usize = 40;

/// The zone a fire falls in, by its slug, for example `chamchamal`. Only the
/// form is checked here: the zones themselves belong to another feature.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZoneSlug(String);

impl ZoneSlug {
    pub fn new(value: String) -> Result<Self, FireError> {
        let well_formed = !value.is_empty()
            && value.len() <= MAX_LENGTH
            && value
                .chars()
                .all(|character| character.is_ascii_lowercase() || character == '-');

        if !well_formed {
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
    fn accepts_a_plain_and_a_hyphenated_slug() {
        assert!(ZoneSlug::new("chamchamal".to_string()).is_ok());
        assert!(ZoneSlug::new("qadir-karam".to_string()).is_ok());
    }

    #[test]
    fn rejects_capitals_digits_spaces_and_other_scripts() {
        assert!(ZoneSlug::new("Chamchamal".to_string()).is_err());
        assert!(ZoneSlug::new("zone1".to_string()).is_err());
        assert!(ZoneSlug::new("qadir karam".to_string()).is_err());
        assert!(ZoneSlug::new("qadir_karam".to_string()).is_err());
        assert!(ZoneSlug::new("چەمچەماڵ".to_string()).is_err());
    }

    #[test]
    fn the_length_is_one_to_forty() {
        assert!(ZoneSlug::new(String::new()).is_err());
        assert!(ZoneSlug::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(ZoneSlug::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }
}
