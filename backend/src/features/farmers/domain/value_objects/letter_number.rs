use chrono::{DateTime, Datelike, Utc};

use crate::{features::farmers::domain::FarmerError, shared::DomainError};

const PREFIX: &str = "JTY";

/// The number printed on a support letter: `JTY-<yyyymm>-<farmer id>-<n>`,
/// where `n` counts that farmer's letters from 1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LetterNumber(String);

impl LetterNumber {
    /// The number of the farmer's `sequence`-th letter, issued at `now`.
    /// The month is the UTC month, like every stored time.
    pub fn issue(now: DateTime<Utc>, farmer_id: i32, sequence: u64) -> Self {
        Self(format!(
            "{PREFIX}-{:04}{:02}-{farmer_id}-{sequence}",
            now.year(),
            now.month()
        ))
    }

    /// A number someone typed or scanned. Only its form is checked.
    pub fn new(value: String) -> Result<Self, FarmerError> {
        let parts: Vec<&str> = value.split('-').collect();
        let all_digits =
            |part: &str| !part.is_empty() && part.chars().all(|digit| digit.is_ascii_digit());

        let well_formed = matches!(
            parts.as_slice(),
            [prefix, month, farmer_id, sequence]
                if *prefix == PREFIX
                    && month.len() == 6
                    && all_digits(month)
                    && farmer_id.len() <= 10
                    && all_digits(farmer_id)
                    && sequence.len() <= 10
                    && all_digits(sequence)
        );

        if !well_formed {
            return Err(DomainError::InvalidValue(
                "A letter number looks like JTY-202610-12-1".to_string(),
            )
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&LetterNumber> for String {
    fn from(value: &LetterNumber) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn a_number_carries_the_month_the_farmer_and_the_count() {
        let now = Utc.with_ymd_and_hms(2026, 3, 9, 12, 0, 0).unwrap();

        assert_eq!(LetterNumber::issue(now, 12, 3).as_str(), "JTY-202603-12-3");
    }

    #[test]
    fn an_issued_number_is_one_that_can_be_looked_up() {
        let issued = LetterNumber::issue(Utc::now(), 2_147_483_647, 4_000_000_000);

        assert_eq!(
            LetterNumber::new(issued.as_str().to_string()).expect("well formed"),
            issued
        );
    }

    #[test]
    fn anything_else_is_not_a_letter_number() {
        for bad in [
            "",
            "JTY",
            "JTY-202610-12",
            "jty-202610-12-1",
            "JTY-2026-12-1",
            "JTY-202610-x-1",
            "JTY-202610-12-1-9",
            "JTY-202610-12-",
            "JTY-202610-12-1%",
        ] {
            assert!(LetterNumber::new(bad.to_string()).is_err(), "{bad}");
        }
    }
}
