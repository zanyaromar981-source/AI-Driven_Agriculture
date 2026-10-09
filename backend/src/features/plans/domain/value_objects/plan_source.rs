use crate::{features::plans::domain::PlanError, shared::DomainError};

const MAX_LENGTH: usize = 120;

/// Where the forecast comes from. It is required: the app prints it under
/// the plan, so the farmer is always told the origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanSource(String);

impl PlanSource {
    pub fn new(value: String) -> Result<Self, PlanError> {
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

impl From<&PlanSource> for String {
    fn from(value: &PlanSource) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        assert_eq!(
            PlanSource::new("  Open-Meteo ".to_string())
                .expect("source")
                .as_str(),
            "Open-Meteo"
        );
    }

    #[test]
    fn a_plan_without_a_source_is_refused() {
        assert!(PlanSource::new(String::new()).is_err());
        assert!(PlanSource::new("   ".to_string()).is_err());
    }

    #[test]
    fn the_limit_is_inclusive_and_counts_characters() {
        assert!(PlanSource::new("س".repeat(MAX_LENGTH)).is_ok());
        assert!(PlanSource::new("س".repeat(MAX_LENGTH + 1)).is_err());
    }
}
