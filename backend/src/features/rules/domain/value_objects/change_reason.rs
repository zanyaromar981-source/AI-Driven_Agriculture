use crate::features::rules::domain::RuleError;

const MIN_LENGTH: usize = 3;
const MAX_LENGTH: usize = 500;

/// Why a staff member changed a rule. Kept with the change for good, so a
/// later reader can tell a decision from a slip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeReason(String);

impl ChangeReason {
    pub fn new(value: String) -> Result<Self, RuleError> {
        let value = value.trim().to_string();

        // Characters, not bytes: a Sorani reason is multi-byte.
        if !(MIN_LENGTH..=MAX_LENGTH).contains(&value.chars().count()) {
            return Err(RuleError::ReasonLength {
                min: MIN_LENGTH,
                max: MAX_LENGTH,
            });
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&ChangeReason> for String {
    fn from(value: &ChangeReason) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_counting() {
        let reason = ChangeReason::new("  late frost this year  ".to_string()).expect("reason");

        assert_eq!(reason.as_str(), "late frost this year");
        assert!(
            ChangeReason::new("  ab  ".to_string()).is_err(),
            "spaces must not pad a reason up to the minimum"
        );
    }

    #[test]
    fn both_limits_are_inclusive() {
        assert!(ChangeReason::new("abc".to_string()).is_ok());
        assert!(ChangeReason::new("ab".to_string()).is_err());
        assert!(ChangeReason::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(ChangeReason::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn a_missing_reason_is_refused() {
        assert!(matches!(
            ChangeReason::new(String::new()),
            Err(RuleError::ReasonLength { min: 3, max: 500 })
        ));
    }

    #[test]
    fn length_is_counted_in_characters_not_bytes() {
        // 500 Sorani letters are 1000 bytes.
        assert!(ChangeReason::new("ڕ".repeat(MAX_LENGTH)).is_ok());
        assert!(ChangeReason::new("ڕ".repeat(MAX_LENGTH + 1)).is_err());
    }
}
