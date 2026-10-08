use crate::{features::alwa::domain::AlwaError, shared::DomainError};

const MAX_LENGTH: usize = 100;

/// Where a price came from, as the data job names it, so a wrong number can
/// be traced back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PriceSource(String);

impl PriceSource {
    pub fn new(value: String) -> Result<Self, AlwaError> {
        let value = value.trim().to_string();

        if value.is_empty() || value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Price source must be 1 to {MAX_LENGTH} characters"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&PriceSource> for String {
    fn from(value: &PriceSource) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_price_must_say_where_it_came_from() {
        assert!(PriceSource::new("alwa-board".to_string()).is_ok());
        assert!(PriceSource::new("  ".to_string()).is_err());
    }

    #[test]
    fn the_limit_is_inclusive() {
        assert!(PriceSource::new("s".repeat(MAX_LENGTH)).is_ok());
        assert!(PriceSource::new("s".repeat(MAX_LENGTH + 1)).is_err());
    }
}
