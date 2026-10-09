use std::fmt;

use chrono::{Datelike, NaiveDate};

use crate::{features::history::domain::HistoryError, shared::DomainError};

const FIRST_YEAR: i32 = 1950;
const LAST_YEAR: i32 = 2100;

/// One calendar month of one year. It travels as `YYYY-MM` and is stored as
/// the first day of the month.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Month {
    year: i32,
    /// 1 to 12.
    month: u32,
}

impl Month {
    pub fn new(year: i32, month: u32) -> Result<Self, HistoryError> {
        if !(FIRST_YEAR..=LAST_YEAR).contains(&year) || !(1..=12).contains(&month) {
            return Err(invalid(format!("{year}-{month}")));
        }

        Ok(Self { year, month })
    }

    /// Reads `YYYY-MM`, and nothing looser: a day, a missing zero or a
    /// slash is a mistake in the caller worth hearing about.
    pub fn parse(value: &str) -> Result<Self, HistoryError> {
        let well_formed = value.len() == 7
            && value.char_indices().all(|(index, character)| {
                if index == 4 {
                    character == '-'
                } else {
                    character.is_ascii_digit()
                }
            });

        if !well_formed {
            return Err(invalid(value.to_string()));
        }

        let year = value[..4].parse().map_err(|_| invalid(value.to_string()))?;
        let month = value[5..].parse().map_err(|_| invalid(value.to_string()))?;

        Self::new(year, month)
    }

    /// The month a day lies in.
    pub fn containing(day: NaiveDate) -> Result<Self, HistoryError> {
        Self::new(day.year(), day.month())
    }

    pub fn year(&self) -> i32 {
        self.year
    }

    /// 1 for January to 12 for December.
    pub fn number(&self) -> u32 {
        self.month
    }

    pub fn first_day(&self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year, self.month, 1)
            .expect("a month between 1 and 12 of a year in range has a first day")
    }

    /// Months since the start of year zero, so two months can be compared
    /// and counted between.
    fn index(&self) -> i32 {
        self.year * 12 + self.month as i32 - 1
    }

    /// The month that many months before this one, stopping at the first
    /// month there is.
    pub fn back(&self, months: u32) -> Self {
        let index = (self.index() - months as i32).max(FIRST_YEAR * 12);

        Self {
            year: index.div_euclid(12),
            month: index.rem_euclid(12) as u32 + 1,
        }
    }

    /// How many months a window from this month to `end` holds, both
    /// counted. Zero or less when `end` is earlier.
    pub fn months_through(&self, end: Month) -> i32 {
        end.index() - self.index() + 1
    }
}

impl fmt::Display for Month {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:04}-{:02}", self.year, self.month)
    }
}

fn invalid(value: String) -> HistoryError {
    DomainError::InvalidValue(format!(
        "Invalid month: {value}. A month is written YYYY-MM, from {FIRST_YEAR} to {LAST_YEAR}"
    ))
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn month(year: i32, month: u32) -> Month {
        Month::new(year, month).expect("month")
    }

    #[test]
    fn reads_and_writes_the_year_and_month_form() {
        let parsed = Month::parse("2016-10").expect("month");

        assert_eq!(parsed, month(2016, 10));
        assert_eq!(parsed.to_string(), "2016-10");
        assert_eq!(month(2024, 3).to_string(), "2024-03");
    }

    #[test]
    fn anything_but_four_digits_a_hyphen_and_two_digits_is_refused() {
        for raw in [
            "2016-1",
            "2016-10-01",
            "2016/10",
            "16-10",
            "2016-13",
            "2016-00",
            "",
            "٢٠١٦-١٠",
        ] {
            assert!(Month::parse(raw).is_err(), "{raw:?} was accepted");
        }
    }

    #[test]
    fn a_year_outside_the_supported_span_is_refused() {
        assert!(Month::parse("1949-12").is_err());
        assert!(Month::parse("1950-01").is_ok());
        assert!(Month::parse("2100-12").is_ok());
        assert!(Month::parse("2101-01").is_err());
    }

    #[test]
    fn it_is_stored_as_its_first_day_and_read_back_from_any_day_in_it() {
        let day = NaiveDate::from_ymd_opt(2024, 2, 29).expect("date");

        assert_eq!(
            month(2024, 2).first_day(),
            NaiveDate::from_ymd_opt(2024, 2, 1).expect("date")
        );
        assert_eq!(Month::containing(day).expect("month"), month(2024, 2));
    }

    #[test]
    fn going_back_crosses_year_ends() {
        assert_eq!(month(2026, 9).back(119), month(2016, 10));
        assert_eq!(month(2026, 1).back(1), month(2025, 12));
        assert_eq!(month(2026, 9).back(0), month(2026, 9));
    }

    #[test]
    fn going_back_stops_at_the_first_supported_month() {
        assert_eq!(month(1950, 3).back(40), month(1950, 1));
    }

    #[test]
    fn a_window_counts_both_of_its_ends() {
        assert_eq!(month(2016, 10).months_through(month(2026, 9)), 120);
        assert_eq!(month(2026, 9).months_through(month(2026, 9)), 1);
        assert!(month(2026, 9).months_through(month(2026, 8)) < 1);
    }

    #[test]
    fn months_sort_by_time() {
        assert!(month(2025, 12) < month(2026, 1));
        assert!(month(2026, 1) < month(2026, 2));
    }
}
