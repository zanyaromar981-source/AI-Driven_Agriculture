use crate::features::zones::domain::{Month, ZoneError};

/// The same calendar month in two different years, which is what the
/// dashboard's "compare with" view puts side by side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct YearComparison {
    month: Month,
    with_month: Month,
}

impl YearComparison {
    pub fn new(year: i32, with: i32, calendar_month: u32) -> Result<Self, ZoneError> {
        if !(1..=12).contains(&calendar_month) {
            return Err(ZoneError::BadCalendarMonth);
        }

        let month_in = |year: i32| {
            Month::new(year, calendar_month).map_err(|_| ZoneError::BadYear {
                min: Month::MIN_YEAR,
                max: Month::MAX_YEAR,
            })
        };

        let month = month_in(year)?;
        let with_month = month_in(with)?;

        if year == with {
            return Err(ZoneError::SameYear);
        }

        Ok(Self { month, with_month })
    }

    /// Reads the three values as they arrive in a query string. A value that
    /// is missing arrives empty and fails like any other that is not a number.
    pub fn parse(year: &str, with: &str, calendar_month: &str) -> Result<Self, ZoneError> {
        let a_year = |raw: &str| {
            raw.trim().parse::<i32>().map_err(|_| ZoneError::BadYear {
                min: Month::MIN_YEAR,
                max: Month::MAX_YEAR,
            })
        };

        let calendar_month = calendar_month
            .trim()
            .parse::<u32>()
            .map_err(|_| ZoneError::BadCalendarMonth)?;

        Self::new(a_year(year)?, a_year(with)?, calendar_month)
    }

    pub fn year(&self) -> i32 {
        self.month.year()
    }

    pub fn with(&self) -> i32 {
        self.with_month.year()
    }

    /// 1 for January to 12 for December.
    pub fn calendar_month(&self) -> u32 {
        self.month.number()
    }

    pub fn month(&self) -> Month {
        self.month
    }

    pub fn with_month(&self) -> Month {
        self.with_month
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_different_years_of_one_calendar_month_compare() {
        let comparison = YearComparison::new(2026, 2025, 3).expect("comparison");

        assert_eq!(comparison.year(), 2026);
        assert_eq!(comparison.with(), 2025);
        assert_eq!(comparison.calendar_month(), 3);
        assert_eq!(String::from(comparison.month()), "2026-03");
        assert_eq!(String::from(comparison.with_month()), "2025-03");
    }

    #[test]
    fn a_year_cannot_be_compared_with_itself() {
        assert!(matches!(
            YearComparison::new(2026, 2026, 3),
            Err(ZoneError::SameYear)
        ));
    }

    #[test]
    fn the_calendar_month_runs_from_one_to_twelve() {
        assert!(YearComparison::new(2026, 2025, 1).is_ok());
        assert!(YearComparison::new(2026, 2025, 12).is_ok());
        assert!(matches!(
            YearComparison::new(2026, 2025, 0),
            Err(ZoneError::BadCalendarMonth)
        ));
        assert!(matches!(
            YearComparison::new(2026, 2025, 13),
            Err(ZoneError::BadCalendarMonth)
        ));
    }

    #[test]
    fn a_year_far_from_today_is_refused() {
        assert!(matches!(
            YearComparison::new(1999, 2025, 3),
            Err(ZoneError::BadYear { .. })
        ));
        assert!(matches!(
            YearComparison::new(2026, 2101, 3),
            Err(ZoneError::BadYear { .. })
        ));
    }

    #[test]
    fn query_values_are_read_with_or_without_a_leading_zero() {
        assert_eq!(
            YearComparison::parse("2026", "2025", "03").expect("comparison"),
            YearComparison::parse("2026", "2025", "3").expect("comparison")
        );
    }

    #[test]
    fn a_missing_or_garbled_query_value_names_what_is_wrong() {
        assert!(matches!(
            YearComparison::parse("", "2025", "3"),
            Err(ZoneError::BadYear { .. })
        ));
        assert!(matches!(
            YearComparison::parse("2026", "last", "3"),
            Err(ZoneError::BadYear { .. })
        ));
        assert!(matches!(
            YearComparison::parse("2026", "2025", ""),
            Err(ZoneError::BadCalendarMonth)
        ));
        assert!(matches!(
            YearComparison::parse("2026", "2025", "march"),
            Err(ZoneError::BadCalendarMonth)
        ));
    }
}
