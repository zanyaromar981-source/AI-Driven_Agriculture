use crate::features::briefs::domain::BriefError;

use super::text;

const MAX_LENGTH: usize = 300;

/// What one point of a brief says, in one language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PointText(String);

impl PointText {
    pub fn new(value: String) -> Result<Self, BriefError> {
        Ok(Self(text::bounded(value, "Point text", MAX_LENGTH)?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&PointText> for String {
    fn from(value: &PointText) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_must_say_something() {
        assert!(PointText::new(String::new()).is_err());
        assert!(PointText::new("   ".to_string()).is_err());
        assert_eq!(
            PointText::new(" Two fires near Qadir Karam ".to_string())
                .expect("text")
                .as_str(),
            "Two fires near Qadir Karam"
        );
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(
            PointText::new("ئ".repeat(MAX_LENGTH)).is_ok(),
            "Sorani text at the limit must not be rejected for its byte length"
        );
        assert!(PointText::new("ئ".repeat(MAX_LENGTH + 1)).is_err());
        assert!(PointText::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }
}
