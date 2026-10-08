use chrono::{Datelike, NaiveDate};

use crate::features::outlooks::domain::OutlookError;

/// The month an outlook was issued in. Kept as the first day of that month.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IssueMonth(NaiveDate);

impl IssueMonth {
    /// Reads `2026-10`. The first day of the month, `2026-10-01`, is taken
    /// as well, because that is how the month is stored.
    pub fn new(value: &str) -> Result<Self, OutlookError> {
        let first_day = match value.len() {
            7 => format!("{value}-01"),
            10 => value.to_string(),
            _ => return Err(OutlookError::InvalidIssueMonth),
        };

        let day: NaiveDate = first_day
            .parse()
            .map_err(|_| OutlookError::InvalidIssueMonth)?;

        Self::from_date(day)
    }

    /// Reconstruct from the stored date, which must be a first of the month.
    pub fn from_date(day: NaiveDate) -> Result<Self, OutlookError> {
        if day.day() != 1 {
            return Err(OutlookError::InvalidIssueMonth);
        }

        Ok(Self(day))
    }

    pub fn first_day(&self) -> NaiveDate {
        self.0
    }
}

impl From<&IssueMonth> for String {
    fn from(value: &IssueMonth) -> Self {
        value.0.format("%Y-%m").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("day")
    }

    #[test]
    fn a_year_and_a_month_are_an_issue_month() {
        let month = IssueMonth::new("2026-10").expect("month");

        assert_eq!(month.first_day(), day(2026, 10, 1));
        assert_eq!(String::from(&month), "2026-10");
    }

    #[test]
    fn the_first_day_of_the_month_is_read_as_that_month() {
        assert_eq!(
            IssueMonth::new("2026-10-01").expect("month"),
            IssueMonth::new("2026-10").expect("month")
        );
    }

    #[test]
    fn any_other_day_of_the_month_is_refused() {
        assert!(matches!(
            IssueMonth::new("2026-10-15"),
            Err(OutlookError::InvalidIssueMonth)
        ));
        assert!(IssueMonth::from_date(day(2026, 10, 2)).is_err());
    }

    #[test]
    fn a_month_that_does_not_exist_is_refused() {
        for bad in [
            "2026-13",
            "2026-00",
            "2026",
            "",
            "october",
            "2026-1",
            "26-10",
            "2026-1-01",
        ] {
            assert!(IssueMonth::new(bad).is_err(), "{bad:?} was accepted");
        }
    }

    #[test]
    fn months_sort_in_time_order() {
        assert!(
            IssueMonth::new("2026-11").expect("month") > IssueMonth::new("2026-09").expect("month")
        );
        assert!(
            IssueMonth::new("2027-01").expect("month") > IssueMonth::new("2026-12").expect("month")
        );
    }
}
