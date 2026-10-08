use crate::{features::zones::domain::ZoneError, shared::DomainError};

const MAX_LENGTH: usize = 100;

/// Which data job or product a reading came from, such as `chirps+modis`.
/// Kept so a number on the dashboard can be traced back to what produced it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadingSource(String);

impl ReadingSource {
    pub fn new(value: String) -> Result<Self, ZoneError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(
                DomainError::InvalidValue("Reading source must not be empty".to_string()).into(),
            );
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Reading source must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<ReadingSource> for String {
    fn from(value: ReadingSource) -> Self {
        value.0
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
        let source = ReadingSource::new("  chirps+modis ".to_string()).expect("source");

        assert_eq!(source.as_str(), "chirps+modis");
    }

    #[test]
    fn a_reading_must_say_where_it_came_from() {
        assert!(ReadingSource::new(String::new()).is_err());
        assert!(ReadingSource::new("   ".to_string()).is_err());
    }

    #[test]
    fn the_limit_is_inclusive() {
        assert!(ReadingSource::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(ReadingSource::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }
}
