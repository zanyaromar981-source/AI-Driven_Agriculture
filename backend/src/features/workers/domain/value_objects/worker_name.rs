use crate::features::workers::domain::WorkerError;

const MAX_LENGTH: usize = 80;

/// The name a worker puts on their card, in their own language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerName(String);

impl WorkerName {
    pub fn new(value: String) -> Result<Self, WorkerError> {
        let value = value.trim().to_string();

        if value.is_empty() || value.chars().count() > MAX_LENGTH {
            return Err(WorkerError::BadName(MAX_LENGTH));
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&WorkerName> for String {
    fn from(value: &WorkerName) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_limit_is_inclusive_and_counts_characters() {
        assert!(WorkerName::new("ک".repeat(MAX_LENGTH)).is_ok());
        assert!(WorkerName::new("ک".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn an_empty_or_blank_name_is_refused() {
        assert!(WorkerName::new(String::new()).is_err());
        assert!(WorkerName::new("   ".to_string()).is_err());
    }

    #[test]
    fn surrounding_whitespace_is_stripped() {
        assert_eq!(
            WorkerName::new(" Azad Karim ".to_string())
                .expect("name")
                .as_str(),
            "Azad Karim"
        );
    }
}
