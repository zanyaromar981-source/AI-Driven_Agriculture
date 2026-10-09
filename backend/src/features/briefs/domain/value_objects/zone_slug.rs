use crate::features::briefs::domain::BriefError;

use super::text;

const MAX_LENGTH: usize = 40;

/// The name a district goes by in URLs and in other slices, such as
/// `chamchamal`. The zones feature owns the districts; a brief only carries
/// the name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ZoneSlug(String);

impl ZoneSlug {
    pub fn new(value: String) -> Result<Self, BriefError> {
        Ok(Self(text::slug(value, "A zone slug", MAX_LENGTH)?))
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
    fn a_district_name_in_lower_case_with_hyphens_is_a_slug() {
        assert_eq!(
            ZoneSlug::new("dashti-hawler".to_string())
                .expect("slug")
                .as_str(),
            "dashti-hawler"
        );
    }

    #[test]
    fn a_name_is_not_a_slug_until_it_is_lowered_and_hyphenated() {
        assert!(ZoneSlug::new("Chamchamal".to_string()).is_err());
        assert!(ZoneSlug::new("qadir karam".to_string()).is_err());
        assert!(ZoneSlug::new(String::new()).is_err());
    }

    #[test]
    fn forty_characters_is_the_longest_slug() {
        assert!(ZoneSlug::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(ZoneSlug::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }
}
