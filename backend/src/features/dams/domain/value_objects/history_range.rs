use chrono::{Months, NaiveDate};

use crate::features::dams::domain::DamError;

const DEFAULT_YEARS: u32 = 10;

/// The days a history request covers, both ends included.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryRange {
    from: NaiveDate,
    to: NaiveDate,
}

impl HistoryRange {
    /// A missing end is today. A missing start is ten years before the end,
    /// so a request with no dates covers the last ten years.
    pub fn new(
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        today: NaiveDate,
    ) -> Result<Self, DamError> {
        let to = to.unwrap_or(today);
        let from = from.unwrap_or_else(|| {
            to.checked_sub_months(Months::new(DEFAULT_YEARS * 12))
                .unwrap_or(NaiveDate::MIN)
        });

        if from > to {
            return Err(DamError::RangeEndsBeforeItStarts);
        }

        Ok(Self { from, to })
    }

    pub fn from(&self) -> NaiveDate {
        self.from
    }

    pub fn to(&self) -> NaiveDate {
        self.to
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("day")
    }

    #[test]
    fn no_dates_means_the_last_ten_years() {
        let range = HistoryRange::new(None, None, day(2026, 10, 8)).expect("range");

        assert_eq!(range.from(), day(2016, 10, 8));
        assert_eq!(range.to(), day(2026, 10, 8));
    }

    #[test]
    fn a_missing_start_is_ten_years_before_the_given_end() {
        let range =
            HistoryRange::new(None, Some(day(2020, 3, 1)), day(2026, 10, 8)).expect("range");

        assert_eq!(range.from(), day(2010, 3, 1));
        assert_eq!(range.to(), day(2020, 3, 1));
    }

    #[test]
    fn a_missing_end_is_today() {
        let range =
            HistoryRange::new(Some(day(1990, 1, 1)), None, day(2026, 10, 8)).expect("range");

        assert_eq!(range.from(), day(1990, 1, 1));
        assert_eq!(range.to(), day(2026, 10, 8));
    }

    #[test]
    fn a_single_day_is_a_range() {
        let one = day(2026, 10, 8);

        assert!(HistoryRange::new(Some(one), Some(one), one).is_ok());
    }

    #[test]
    fn a_range_that_ends_before_it_starts_is_refused() {
        let result = HistoryRange::new(
            Some(day(2026, 2, 1)),
            Some(day(2026, 1, 1)),
            day(2026, 10, 8),
        );

        assert!(matches!(result, Err(DamError::RangeEndsBeforeItStarts)));
    }
}
