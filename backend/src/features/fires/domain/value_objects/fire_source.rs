use crate::{features::fires::domain::FireError, shared::DomainError};

const MAX_LENGTH: usize = 120;

/// Where the detection comes from, for example the satellite instrument. The
/// dashboard shows it next to the fire so nobody takes it for a ground report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FireSource(String);

impl FireSource {
    pub fn new(value: String) -> Result<Self, FireError> {
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

impl From<&FireSource> for String {
    fn from(value: &FireSource) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        assert_eq!(
            FireSource::new("  NASA FIRMS VIIRS ".to_string())
                .expect("source")
                .as_str(),
            "NASA FIRMS VIIRS"
        );
    }

    #[test]
    fn a_fire_without_a_source_is_refused() {
        assert!(FireSource::new("   ".to_string()).is_err());
    }

    #[test]
    fn the_limit_is_inclusive() {
        assert!(FireSource::new("s".repeat(MAX_LENGTH)).is_ok());
        assert!(FireSource::new("s".repeat(MAX_LENGTH + 1)).is_err());
    }
}
