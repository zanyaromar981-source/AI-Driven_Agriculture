use crate::{features::zones::domain::ZoneError, shared::DomainError};

const MAX_LENGTH: usize = 60;

/// The name a zone or sub-zone goes by in URLs and in other slices: its
/// English name in lower case with hyphens for spaces, such as
/// `qadir-karam`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ZoneSlug(String);

impl ZoneSlug {
    pub fn new(value: String) -> Result<Self, ZoneError> {
        if value.is_empty() || value.len() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "A zone slug must be 1 to {MAX_LENGTH} characters"
            ))
            .into());
        }

        let allowed = |character: char| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        };

        if !value.chars().all(allowed) || value.starts_with('-') || value.ends_with('-') {
            return Err(DomainError::InvalidValue(
                "A zone slug is lower-case letters and digits joined by hyphens".to_string(),
            )
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<ZoneSlug> for String {
    fn from(value: ZoneSlug) -> Self {
        value.0
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
    fn the_seeded_slugs_are_valid() {
        for slug in ["chamchamal", "qadir-karam", "qadir-karam", "markaz-zakho"] {
            assert!(ZoneSlug::new(slug.to_string()).is_ok(), "{slug}");
        }
    }

    #[test]
    fn a_name_is_not_a_slug_until_it_is_lowered_and_hyphenated() {
        assert!(ZoneSlug::new("Qadir Karam".to_string()).is_err());
        assert!(ZoneSlug::new("qadir karam".to_string()).is_err());
        assert!(ZoneSlug::new("Chamchamal".to_string()).is_err());
    }

    #[test]
    fn a_hyphen_only_joins() {
        assert!(ZoneSlug::new("-kalar".to_string()).is_err());
        assert!(ZoneSlug::new("kalar-".to_string()).is_err());
    }

    #[test]
    fn an_empty_or_over_long_slug_is_refused() {
        assert!(ZoneSlug::new(String::new()).is_err());
        assert!(ZoneSlug::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(ZoneSlug::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn it_is_not_trimmed_because_a_url_segment_has_no_spare_spaces() {
        assert!(ZoneSlug::new(" kalar".to_string()).is_err());
    }
}
