use chrono::{Datelike, Months, NaiveDate};

use crate::features::zones::domain::ZoneError;

/// One calendar month of one year, the grain every reading is kept at. It is
/// stored as the first day of the month and written `YYYY-MM` on the wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Month(NaiveDate);

impl Month {
    pub const MIN_YEAR: i32 = 2000;
    pub const MAX_YEAR: i32 = 2100;

    pub fn new(year: i32, number: u32) -> Result<Self, ZoneError> {
        if !(Self::MIN_YEAR..=Self::MAX_YEAR).contains(&year) {
            return Err(Self::bad());
        }

        NaiveDate::from_ymd_opt(year, number, 1)
            .map(Self)
            .ok_or_else(Self::bad)
    }

    /// Reads `YYYY-MM`. Anything looser (`2026-3`, `2026-03-01`) is refused
    /// so that one month has exactly one spelling in a URL.
    pub fn parse(value: &str) -> Result<Self, ZoneError> {
        let (year, number) = value.split_once('-').ok_or_else(Self::bad)?;

        let digits = |part: &str| part.chars().all(|character| character.is_ascii_digit());

        if year.len() != 4 || number.len() != 2 || !digits(year) || !digits(number) {
            return Err(Self::bad());
        }

        Self::new(
            year.parse().map_err(|_| Self::bad())?,
            number.parse().map_err(|_| Self::bad())?,
        )
    }

    /// The month a day falls in.
    pub fn containing(day: NaiveDate) -> Self {
        Self(day.with_day(1).unwrap_or(day))
    }

    pub fn year(&self) -> i32 {
        self.0.year()
    }

    /// 1 for January to 12 for December.
    pub fn number(&self) -> u32 {
        self.0.month()
    }

    pub fn first_day(&self) -> NaiveDate {
        self.0
    }

    /// The same calendar month one year before.
    pub fn a_year_earlier(&self) -> Option<Self> {
        NaiveDate::from_ymd_opt(self.year() - 1, self.number(), 1).map(Self)
    }

    /// The month `count` months before this one. It is used as a bound of a
    /// range, so it may fall before `MIN_YEAR`.
    pub fn months_earlier(&self, count: u32) -> Option<Self> {
        self.0.checked_sub_months(Months::new(count)).map(Self)
    }

    fn bad() -> ZoneError {
        ZoneError::BadMonth {
            min_year: Self::MIN_YEAR,
            max_year: Self::MAX_YEAR,
        }
    }
}

impl From<&Month> for String {
    fn from(value: &Month) -> Self {
        value.0.format("%Y-%m").to_string()
    }
}

impl From<Month> for String {
    fn from(value: Month) -> Self {
        String::from(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_month_is_read_and_written_the_same_way() {
        let month = Month::parse("2026-03").expect("month");

        assert_eq!(month.year(), 2026);
        assert_eq!(month.number(), 3);
        assert_eq!(String::from(month), "2026-03");
    }

    #[test]
    fn it_is_kept_as_the_first_day_of_the_month() {
        let month = Month::parse("2026-03").expect("month");

        assert_eq!(
            month.first_day(),
            NaiveDate::from_ymd_opt(2026, 3, 1).expect("date")
        );
    }

    #[test]
    fn a_month_has_exactly_one_spelling() {
        for loose in [
            "2026-3",
            "2026-03-01",
            "2026/03",
            "202603",
            "26-03",
            "",
            "march",
            "+026-03",
        ] {
            assert!(
                matches!(Month::parse(loose), Err(ZoneError::BadMonth { .. })),
                "{loose:?} was accepted"
            );
        }
    }

    #[test]
    fn a_month_number_outside_the_calendar_is_refused() {
        assert!(Month::parse("2026-00").is_err());
        assert!(Month::parse("2026-13").is_err());
        assert!(Month::parse("2026-12").is_ok());
    }

    #[test]
    fn a_year_far_from_today_is_refused() {
        assert!(Month::parse("1999-12").is_err());
        assert!(Month::parse("2000-01").is_ok());
        assert!(Month::parse("2100-12").is_ok());
        assert!(Month::parse("2101-01").is_err());
    }

    #[test]
    fn a_year_earlier_keeps_the_calendar_month() {
        let month = Month::parse("2026-03").expect("month");

        assert_eq!(
            month.a_year_earlier().map(String::from),
            Some("2025-03".to_string())
        );
    }

    #[test]
    fn months_earlier_crosses_the_turn_of_the_year() {
        let month = Month::parse("2026-03").expect("month");

        assert_eq!(
            month.months_earlier(23).map(String::from),
            Some("2024-04".to_string())
        );
        assert_eq!(month.months_earlier(0), Some(month));
    }

    #[test]
    fn any_day_belongs_to_its_month() {
        let day = NaiveDate::from_ymd_opt(2026, 10, 8).expect("date");

        assert_eq!(String::from(Month::containing(day)), "2026-10");
    }

    #[test]
    fn months_sort_oldest_first() {
        let mut months = vec![
            Month::parse("2026-03").expect("month"),
            Month::parse("2024-11").expect("month"),
            Month::parse("2025-03").expect("month"),
        ];

        months.sort();

        assert_eq!(
            months.into_iter().map(String::from).collect::<Vec<_>>(),
            vec!["2024-11", "2025-03", "2026-03"]
        );
    }
}
