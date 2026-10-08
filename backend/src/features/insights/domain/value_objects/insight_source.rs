use crate::{features::insights::domain::InsightError, shared::DomainError};

const MAX_LENGTH: usize = 120;

/// Where the numbers of a reading come from, for example the satellite or the
/// weather service. It is required: the farmer is always told the origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InsightSource(String);

impl InsightSource {
    pub fn new(value: String) -> Result<Self, InsightError> {
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

impl From<&InsightSource> for String {
    fn from(value: &InsightSource) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        assert_eq!(
            InsightSource::new("  Sentinel-2 ".to_string())
                .expect("source")
                .as_str(),
            "Sentinel-2"
        );
    }

    #[test]
    fn a_reading_without_a_source_is_refused() {
        assert!(InsightSource::new(String::new()).is_err());
        assert!(InsightSource::new("   ".to_string()).is_err());
    }

    #[test]
    fn the_limit_is_inclusive_and_counts_characters() {
        assert!(InsightSource::new("س".repeat(MAX_LENGTH)).is_ok());
        assert!(InsightSource::new("س".repeat(MAX_LENGTH + 1)).is_err());
    }
}
