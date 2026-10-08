use crate::{features::staff::domain::StaffError, shared::DomainError};

const MAX_LENGTH: usize = 254;

/// The address a staff member signs in with. It is kept in lower case so
/// that the unique index treats `A@x.org` and `a@x.org` as one account.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaffEmail(String);

impl StaffEmail {
    pub fn new(value: String) -> Result<Self, StaffError> {
        let value = value.trim().to_lowercase();

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Email must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        let well_formed = match value.split_once('@') {
            Some((local, domain)) => {
                !local.is_empty()
                    && !domain.is_empty()
                    && !domain.contains('@')
                    && !value
                        .chars()
                        .any(|character| character.is_whitespace() || character.is_control())
            }
            None => false,
        };

        if !well_formed {
            return Err(
                DomainError::InvalidValue("Email is not a valid address".to_string()).into(),
            );
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&StaffEmail> for String {
    fn from(value: &StaffEmail) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_email_is_stored_trimmed_and_in_lower_case() {
        assert_eq!(
            StaffEmail::new("  Hiwa.K@Example.ORG ".to_string())
                .expect("email")
                .as_str(),
            "hiwa.k@example.org"
        );
    }

    #[test]
    fn an_address_needs_one_at_sign_with_text_on_both_sides() {
        for bad in [
            "",
            "hiwa",
            "@example.org",
            "hiwa@",
            "a@b@c",
            "hi wa@example.org",
        ] {
            assert!(StaffEmail::new(bad.to_string()).is_err(), "{bad:?} passed");
        }
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        let domain = "@example.org";
        let fits = format!("{}{domain}", "ک".repeat(MAX_LENGTH - domain.len()));
        let too_long = format!("{}{domain}", "ک".repeat(MAX_LENGTH - domain.len() + 1));

        assert!(StaffEmail::new(fits).is_ok());
        assert!(StaffEmail::new(too_long).is_err());
    }
}
