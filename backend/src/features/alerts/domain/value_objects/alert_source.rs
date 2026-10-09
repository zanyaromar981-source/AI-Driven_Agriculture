use crate::{features::alerts::domain::AlertError, shared::DomainError};

const MAX_LENGTH: usize = 120;

/// Which job made an alert and from what, for example
/// `farm_alerts.py: NASA FIRMS detection near the farm`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlertSource(String);

impl AlertSource {
    pub fn new(value: String) -> Result<Self, AlertError> {
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

impl From<&AlertSource> for String {
    fn from(value: &AlertSource) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        assert_eq!(
            AlertSource::new("  farm_alerts.py ".to_string())
                .expect("source")
                .as_str(),
            "farm_alerts.py"
        );
    }

    #[test]
    fn an_alert_without_a_source_is_refused() {
        assert!(AlertSource::new("   ".to_string()).is_err());
    }

    #[test]
    fn the_limit_is_inclusive_and_counts_characters() {
        assert!(AlertSource::new("س".repeat(MAX_LENGTH)).is_ok());
        assert!(AlertSource::new("س".repeat(MAX_LENGTH + 1)).is_err());
    }
}
