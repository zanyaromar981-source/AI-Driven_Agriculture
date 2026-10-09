use crate::features::briefs::domain::BriefError;

use super::text;

const MAX_LENGTH: usize = 120;

/// The one line a brief opens with, in one language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Headline(String);

impl Headline {
    pub fn new(value: String) -> Result<Self, BriefError> {
        Ok(Self(text::bounded(value, "Headline", MAX_LENGTH)?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&Headline> for String {
    fn from(value: &Headline) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_must_say_something() {
        assert!(Headline::new(String::new()).is_err());
        assert!(Headline::new("   ".to_string()).is_err());
        assert_eq!(
            Headline::new(" Rain is two weeks late ".to_string())
                .expect("text")
                .as_str(),
            "Rain is two weeks late"
        );
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(
            Headline::new("ئ".repeat(MAX_LENGTH)).is_ok(),
            "Sorani text at the limit must not be rejected for its byte length"
        );
        assert!(Headline::new("ئ".repeat(MAX_LENGTH + 1)).is_err());
        assert!(Headline::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }
}
