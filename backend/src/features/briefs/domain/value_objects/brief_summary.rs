use crate::features::briefs::domain::BriefError;

use super::text;

const MAX_LENGTH: usize = 2_000;

/// The few paragraphs of a brief, in one language. The nightly job writes
/// it; the backend only keeps it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BriefSummary(String);

impl BriefSummary {
    pub fn new(value: String) -> Result<Self, BriefError> {
        Ok(Self(text::bounded(value, "Summary", MAX_LENGTH)?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&BriefSummary> for String {
    fn from(value: &BriefSummary) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_must_say_something() {
        assert!(BriefSummary::new(String::new()).is_err());
        assert!(BriefSummary::new("   ".to_string()).is_err());
        assert_eq!(
            BriefSummary::new(" Dukan stands at 38% of its capacity. ".to_string())
                .expect("text")
                .as_str(),
            "Dukan stands at 38% of its capacity."
        );
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(
            BriefSummary::new("ئ".repeat(MAX_LENGTH)).is_ok(),
            "Sorani text at the limit must not be rejected for its byte length"
        );
        assert!(BriefSummary::new("ئ".repeat(MAX_LENGTH + 1)).is_err());
        assert!(BriefSummary::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }
}
