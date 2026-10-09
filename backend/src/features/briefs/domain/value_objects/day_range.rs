use chrono::{Days, NaiveDate};
use getset::Getters;

use crate::{features::briefs::domain::BriefError, shared::DomainError};

const DEFAULT_DAYS: u64 = 14;
const MAX_DAYS: i64 = 92;

/// The days a list of briefs covers, both ends included.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct DayRange {
    from: NaiveDate,
    to: NaiveDate,
}

impl DayRange {
    pub fn new(from: NaiveDate, to: NaiveDate) -> Result<Self, BriefError> {
        if from > to {
            return Err(BriefError::RangeEndsBeforeItStarts);
        }

        if (to - from).num_days() + 1 > MAX_DAYS {
            return Err(BriefError::RangeTooLong { max: MAX_DAYS });
        }

        Ok(Self { from, to })
    }

    /// Fills in the ends a caller left out. With no end the range runs to
    /// the day after `today`: the server's clock is UTC and the region is
    /// ahead of it, so the brief written at local midnight carries a day the
    /// server has not reached yet and must not be cut off. With no start it
    /// reaches back far enough to hold the last fourteen days.
    pub fn resolve(
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        today: NaiveDate,
    ) -> Result<Self, BriefError> {
        let out_of_calendar =
            || DomainError::InvalidValue("The range is outside the calendar".to_string());

        let from = match from {
            Some(from) => from,
            None => to
                .unwrap_or(today)
                .checked_sub_days(Days::new(DEFAULT_DAYS - 1))
                .ok_or_else(out_of_calendar)?,
        };

        let to = match to {
            Some(to) => to,
            None => today
                .checked_add_days(Days::new(1))
                .ok_or_else(out_of_calendar)?,
        };

        Self::new(from, to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, month, day).expect("date")
    }

    #[test]
    fn with_nothing_asked_it_holds_the_last_fourteen_days_and_tomorrow() {
        let range = DayRange::resolve(None, None, day(10, 9)).expect("range");

        assert_eq!(
            *range.from(),
            day(9, 26),
            "9 October back to 26 September is 14 days"
        );
        assert_eq!(
            *range.to(),
            day(10, 10),
            "a brief dated by the region's clock may be a day ahead of UTC"
        );
    }

    #[test]
    fn an_end_alone_reaches_fourteen_days_back_from_it() {
        let range = DayRange::resolve(None, Some(day(8, 31)), day(10, 9)).expect("range");

        assert_eq!(*range.from(), day(8, 18));
        assert_eq!(*range.to(), day(8, 31));
    }

    #[test]
    fn a_start_alone_runs_up_to_now() {
        let range = DayRange::resolve(Some(day(9, 1)), None, day(10, 9)).expect("range");

        assert_eq!(*range.from(), day(9, 1));
        assert_eq!(*range.to(), day(10, 10));
    }

    #[test]
    fn one_day_is_a_range() {
        assert!(DayRange::new(day(10, 9), day(10, 9)).is_ok());
    }

    #[test]
    fn a_range_that_ends_before_it_starts_is_refused() {
        assert!(matches!(
            DayRange::new(day(10, 9), day(10, 8)),
            Err(BriefError::RangeEndsBeforeItStarts)
        ));
    }

    #[test]
    fn ninety_two_days_is_the_longest_range() {
        // 1 July to 30 September is 31 + 31 + 30 days.
        assert!(DayRange::new(day(7, 1), day(9, 30)).is_ok());
        assert!(matches!(
            DayRange::new(day(7, 1), day(10, 1)),
            Err(BriefError::RangeTooLong { max: 92 })
        ));
    }

    #[test]
    fn a_start_long_ago_without_an_end_is_too_long_rather_than_cut_short() {
        assert!(matches!(
            DayRange::resolve(Some(day(1, 1)), None, day(10, 9)),
            Err(BriefError::RangeTooLong { .. })
        ));
    }

    #[test]
    fn an_end_at_the_edge_of_the_calendar_is_an_error_not_a_crash() {
        assert!(DayRange::resolve(None, Some(NaiveDate::MIN), day(10, 9)).is_err());
    }
}
