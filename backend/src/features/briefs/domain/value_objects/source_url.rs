use crate::{features::briefs::domain::BriefError, shared::DomainError};

const MAX_LENGTH: usize = 500;

/// Where a page the nightly job read can be opened. Only web addresses: the
/// dashboard renders it as a link, so nothing like `javascript:` may pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceUrl(String);

impl SourceUrl {
    pub fn new(value: String) -> Result<Self, BriefError> {
        let value = value.trim().to_string();

        let rest = value
            .strip_prefix("https://")
            .or_else(|| value.strip_prefix("http://"));

        if !rest.is_some_and(|rest| !rest.is_empty()) || value.chars().any(char::is_whitespace) {
            return Err(DomainError::InvalidValue(
                "Source url must start with https:// or http://".to_string(),
            )
            .into());
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Source url must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&SourceUrl> for String {
    fn from(value: &SourceUrl) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_web_address_is_accepted_with_either_scheme() {
        assert!(SourceUrl::new("https://example.org/rain".to_string()).is_ok());
        assert!(SourceUrl::new("http://example.org".to_string()).is_ok());
    }

    #[test]
    fn anything_that_is_not_a_web_address_is_refused() {
        for bad in [
            "",
            "example.org",
            "ftp://example.org",
            "javascript:alert(1)",
            "HTTPS://example.org",
            "https://",
            "https://example.org/a b",
        ] {
            assert!(SourceUrl::new(bad.to_string()).is_err(), "{bad}");
        }
    }

    #[test]
    fn five_hundred_characters_is_the_longest_address() {
        let prefix = "https://example.org/";

        assert!(
            SourceUrl::new(format!("{prefix}{}", "a".repeat(MAX_LENGTH - prefix.len()))).is_ok()
        );
        assert!(
            SourceUrl::new(format!(
                "{prefix}{}",
                "a".repeat(MAX_LENGTH - prefix.len() + 1)
            ))
            .is_err()
        );
    }
}
