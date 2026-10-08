use crate::{features::alwa::domain::AlwaError, shared::DomainError};

use super::is_slug;

/// The name an alwa is addressed by in a URL, for example `sulaymaniyah`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MarketSlug(String);

impl MarketSlug {
    pub fn new(value: String) -> Result<Self, AlwaError> {
        if !is_slug(&value) {
            return Err(DomainError::InvalidValue(
                "Market must be 1 to 40 lower-case letters and hyphens".to_string(),
            )
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&MarketSlug> for String {
    fn from(value: &MarketSlug) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_slug_of_a_real_alwa() {
        assert_eq!(
            MarketSlug::new("sulaymaniyah".to_string())
                .expect("slug")
                .as_str(),
            "sulaymaniyah"
        );
    }

    #[test]
    fn rejects_anything_that_is_not_a_slug() {
        assert!(MarketSlug::new(String::new()).is_err());
        assert!(MarketSlug::new("Erbil".to_string()).is_err());
        assert!(MarketSlug::new("هەولێر".to_string()).is_err());
        assert!(MarketSlug::new("a".repeat(41)).is_err());
    }
}
