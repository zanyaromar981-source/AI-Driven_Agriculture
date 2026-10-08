use crate::{features::alwa::domain::AlwaError, shared::DomainError};

const MAX_DAYS: u32 = 90;

/// How many days of price history to show, today included.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryDays(u32);

impl HistoryDays {
    pub const DEFAULT: HistoryDays = HistoryDays(7);

    pub fn new(value: u32) -> Result<Self, AlwaError> {
        if !(1..=MAX_DAYS).contains(&value) {
            return Err(DomainError::InvalidValue(format!("Days must be 1 to {MAX_DAYS}")).into());
        }

        Ok(Self(value))
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_week_is_the_default() {
        assert_eq!(HistoryDays::DEFAULT.value(), 7);
    }

    #[test]
    fn accepts_one_day_up_to_ninety() {
        assert!(HistoryDays::new(1).is_ok());
        assert!(HistoryDays::new(MAX_DAYS).is_ok());
    }

    #[test]
    fn rejects_no_days_and_more_than_ninety() {
        assert!(HistoryDays::new(0).is_err());
        assert!(HistoryDays::new(MAX_DAYS + 1).is_err());
    }
}
