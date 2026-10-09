use chrono::{Duration, NaiveDate};

use crate::{features::alerts::domain::AlertError, shared::DomainError};

const MIN_DAYS: i64 = 1;
const MAX_DAYS: i64 = 90;
const DEFAULT_DAYS: i64 = 30;

/// How many days back a list of alerts reaches, 1 to 90.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryDays(i64);

impl HistoryDays {
    pub fn new(value: i64) -> Result<Self, AlertError> {
        if !(MIN_DAYS..=MAX_DAYS).contains(&value) {
            return Err(DomainError::InvalidValue(format!(
                "Days must be {MIN_DAYS} to {MAX_DAYS}"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn days(&self) -> i64 {
        self.0
    }

    /// The earliest alert day the list includes. Alerts for days still to
    /// come are always included, so there is no upper end.
    pub fn first_day(&self, today: NaiveDate) -> NaiveDate {
        today - Duration::days(self.0)
    }
}

impl Default for HistoryDays {
    fn default() -> Self {
        Self(DEFAULT_DAYS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thirty_days_when_nothing_is_asked_for() {
        assert_eq!(HistoryDays::default().days(), 30);
    }

    #[test]
    fn one_to_ninety_days_are_accepted_and_nothing_else() {
        assert!(HistoryDays::new(1).is_ok());
        assert!(HistoryDays::new(90).is_ok());
        assert!(HistoryDays::new(0).is_err());
        assert!(HistoryDays::new(91).is_err());
        assert!(HistoryDays::new(-5).is_err());
    }

    #[test]
    fn the_first_day_is_that_many_days_before_today() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 9).expect("day");

        assert_eq!(
            HistoryDays::new(7).expect("days").first_day(today),
            NaiveDate::from_ymd_opt(2026, 10, 2).expect("day")
        );
    }
}
