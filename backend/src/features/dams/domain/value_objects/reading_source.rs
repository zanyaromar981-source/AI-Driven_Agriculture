use crate::{features::dams::domain::DamError, shared::DomainError};

const MAX_LENGTH: usize = 100;

/// Where a reading came from, as the data job names it. Kept with every
/// reading so a number on the dashboard can be traced back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadingSource(String);

impl ReadingSource {
    pub fn new(value: String) -> Result<Self, DamError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(DomainError::InvalidValue("source must not be empty".to_string()).into());
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "source must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&ReadingSource> for String {
    fn from(value: &ReadingSource) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        let source = ReadingSource::new("  sentinel-2  ".to_string()).expect("source");

        assert_eq!(source.as_str(), "sentinel-2");
    }

    #[test]
    fn a_reading_without_a_source_is_refused() {
        assert!(ReadingSource::new(String::new()).is_err());
        assert!(ReadingSource::new("   ".to_string()).is_err());
    }

    #[test]
    fn the_limit_is_inclusive() {
        assert!(ReadingSource::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(ReadingSource::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }
}
