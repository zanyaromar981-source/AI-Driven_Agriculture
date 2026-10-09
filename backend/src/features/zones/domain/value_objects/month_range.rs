use crate::features::zones::domain::{Month, ZoneError};

/// The months of stored readings an editing screen asks for, both ends
/// included.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonthRange {
    from: Month,
    to: Month,
}

impl MonthRange {
    /// How many months are shown when no bound is given.
    pub const DEFAULT_MONTHS: u32 = 24;

    /// A missing `to` is the current month, and a missing `from` is the
    /// month that makes the range `DEFAULT_MONTHS` long.
    pub fn new(from: Option<Month>, to: Option<Month>, current: Month) -> Result<Self, ZoneError> {
        let to = to.unwrap_or(current);
        let from = from
            .or_else(|| to.months_earlier(Self::DEFAULT_MONTHS - 1))
            .unwrap_or(to);

        if from > to {
            return Err(ZoneError::FromAfterTo);
        }

        Ok(Self { from, to })
    }

    pub fn from(&self) -> Month {
        self.from
    }

    pub fn to(&self) -> Month {
        self.to
    }

    pub fn contains(&self, month: Month) -> bool {
        self.from <= month && month <= self.to
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn month(value: &str) -> Month {
        Month::parse(value).expect("month")
    }

    fn written(range: MonthRange) -> (String, String) {
        (String::from(range.from()), String::from(range.to()))
    }

    #[test]
    fn with_no_bounds_it_is_the_last_24_months_ending_now() {
        let range = MonthRange::new(None, None, month("2026-10")).expect("range");

        assert_eq!(
            written(range),
            ("2024-11".to_string(), "2026-10".to_string())
        );
    }

    #[test]
    fn a_lone_to_is_the_end_of_24_months() {
        let range = MonthRange::new(None, Some(month("2025-12")), month("2026-10")).expect("range");

        assert_eq!(
            written(range),
            ("2024-01".to_string(), "2025-12".to_string())
        );
    }

    #[test]
    fn a_lone_from_runs_up_to_the_current_month() {
        let range = MonthRange::new(Some(month("2020-01")), None, month("2026-10")).expect("range");

        assert_eq!(
            written(range),
            ("2020-01".to_string(), "2026-10".to_string())
        );
    }

    #[test]
    fn one_month_is_a_range() {
        let range = MonthRange::new(
            Some(month("2026-03")),
            Some(month("2026-03")),
            month("2026-10"),
        )
        .expect("range");

        assert!(range.contains(month("2026-03")));
        assert!(!range.contains(month("2026-02")));
        assert!(!range.contains(month("2026-04")));
    }

    #[test]
    fn a_from_after_the_to_is_refused() {
        assert!(matches!(
            MonthRange::new(
                Some(month("2026-04")),
                Some(month("2026-03")),
                month("2026-10")
            ),
            Err(ZoneError::FromAfterTo)
        ));
        assert!(
            matches!(
                MonthRange::new(Some(month("2027-01")), None, month("2026-10")),
                Err(ZoneError::FromAfterTo)
            ),
            "a missing `to` is the current month, which a later `from` is after"
        );
    }
}
