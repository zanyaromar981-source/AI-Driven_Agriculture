use crate::{features::water::domain::WaterError, shared::DomainError};

const MAX_LENGTH: usize = 40;

/// The name a zone goes by, for example `chamchamal`. Only its form is
/// checked here: the zones themselves belong to another slice.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ZoneSlug(String);

impl ZoneSlug {
    pub fn new(value: String) -> Result<Self, WaterError> {
        let well_formed = !value.is_empty()
            && value.len() <= MAX_LENGTH
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'-');

        if !well_formed {
            return Err(DomainError::InvalidValue(format!(
                "A zone slug is 1 to {MAX_LENGTH} lower-case letters and hyphens"
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
    fn lower_case_letters_and_hyphens_are_a_slug() {
        assert!(ZoneSlug::new("chamchamal".to_string()).is_ok());
        assert_eq!(
            ZoneSlug::new("qadir-karam".to_string())
                .expect("slug")
                .as_str(),
            "qadir-karam"
        );
    }

    #[test]
    fn anything_else_is_not_a_slug() {
        for bad in ["", "Makhmur", "qadir karam", "zone_1", "zone7", "مەخموور"] {
            assert!(
                ZoneSlug::new(bad.to_string()).is_err(),
                "{bad:?} was accepted"
            );
        }
    }

    #[test]
    fn the_length_limit_is_inclusive() {
        assert!(ZoneSlug::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(ZoneSlug::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }
}
