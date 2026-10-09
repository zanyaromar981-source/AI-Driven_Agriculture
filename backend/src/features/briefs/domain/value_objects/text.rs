use crate::{features::briefs::domain::BriefError, shared::DomainError};

/// The rule every text of a brief shares: something is written, and it is
/// no longer than its limit. Characters are counted, not bytes, because a
/// Sorani letter takes two bytes.
pub(super) fn bounded(value: String, name: &str, max: usize) -> Result<String, BriefError> {
    let value = value.trim().to_string();

    if value.is_empty() {
        return Err(DomainError::InvalidValue(format!("{name} must not be empty")).into());
    }

    if value.chars().count() > max {
        return Err(
            DomainError::InvalidValue(format!("{name} must be {max} characters max")).into(),
        );
    }

    Ok(value)
}

/// The shape of a zone slug: lower-case letters and digits joined by
/// hyphens. It is not trimmed, because a URL segment has no spare spaces.
pub(super) fn slug(value: String, name: &str, max: usize) -> Result<String, BriefError> {
    if value.is_empty() || value.len() > max {
        return Err(
            DomainError::InvalidValue(format!("{name} must be 1 to {max} characters")).into(),
        );
    }

    let allowed = |character: char| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
    };

    if !value.chars().all(allowed) || value.starts_with('-') || value.ends_with('-') {
        return Err(DomainError::InvalidValue(format!(
            "{name} is lower-case letters and digits joined by hyphens"
        ))
        .into());
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_counting() {
        assert_eq!(
            bounded("  rain  ".to_string(), "Text", 4).expect("text"),
            "rain"
        );
    }

    #[test]
    fn only_whitespace_is_empty() {
        assert!(bounded(" \n ".to_string(), "Text", 10).is_err());
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(
            bounded("ئ".repeat(10), "Text", 10).is_ok(),
            "ten Sorani letters are twenty bytes and still ten characters"
        );
        assert!(bounded("ئ".repeat(11), "Text", 10).is_err());
    }

    #[test]
    fn a_slug_is_lower_case_letters_digits_and_joining_hyphens() {
        for good in ["chamchamal", "dashti-hawler", "zone-2", "a"] {
            assert!(slug(good.to_string(), "Slug", 40).is_ok(), "{good}");
        }

        for bad in [
            "",
            "Chamchamal",
            "qadir karam",
            "-kalar",
            "kalar-",
            " kalar",
            "کەلار",
        ] {
            assert!(slug(bad.to_string(), "Slug", 40).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_slug_longer_than_its_limit_is_refused() {
        assert!(slug("a".repeat(40), "Slug", 40).is_ok());
        assert!(slug("a".repeat(41), "Slug", 40).is_err());
    }
}
