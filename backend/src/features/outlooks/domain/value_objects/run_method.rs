use crate::{features::outlooks::domain::OutlookError, shared::DomainError};

const MAX_LENGTH: usize = 200;

/// The name of the method a run of outlooks was made with, as the data job
/// gives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunMethod(String);

impl RunMethod {
    pub fn new(value: String) -> Result<Self, OutlookError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(DomainError::InvalidValue("method must not be empty".to_string()).into());
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "method must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&RunMethod> for String {
    fn from(value: &RunMethod) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        let method = RunMethod::new(" analog years ".to_string()).expect("method");

        assert_eq!(method.as_str(), "analog years");
    }

    #[test]
    fn a_run_without_a_method_is_refused() {
        assert!(RunMethod::new(String::new()).is_err());
        assert!(RunMethod::new("  ".to_string()).is_err());
    }

    #[test]
    fn the_limit_is_inclusive() {
        assert!(RunMethod::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(RunMethod::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }
}
