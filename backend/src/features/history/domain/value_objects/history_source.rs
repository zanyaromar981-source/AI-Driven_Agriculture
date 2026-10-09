use crate::{features::history::domain::HistoryError, shared::DomainError};

const MAX_LENGTH: usize = 200;

/// In plain words what a series measures, who measured it and at what
/// resolution. It is required: a farmer is always told where a number comes
/// from and how wide an area it speaks for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistorySource(String);

impl HistorySource {
    pub fn new(value: String) -> Result<Self, HistoryError> {
        let value = value.trim().to_string();

        if value.is_empty() || value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Source must be 1 to {MAX_LENGTH} characters"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&HistorySource> for String {
    fn from(value: &HistorySource) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        assert_eq!(
            HistorySource::new("  ERA5-Land via Open-Meteo ".to_string())
                .expect("source")
                .as_str(),
            "ERA5-Land via Open-Meteo"
        );
    }

    #[test]
    fn a_series_without_a_source_is_refused() {
        assert!(HistorySource::new(String::new()).is_err());
        assert!(HistorySource::new("   ".to_string()).is_err());
    }

    #[test]
    fn the_limit_is_inclusive_and_counts_characters() {
        assert!(HistorySource::new("س".repeat(MAX_LENGTH)).is_ok());
        assert!(HistorySource::new("س".repeat(MAX_LENGTH + 1)).is_err());
    }
}
