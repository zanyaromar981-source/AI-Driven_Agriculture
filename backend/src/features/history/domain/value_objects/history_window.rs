use chrono::{DateTime, Utc};

use crate::features::history::domain::{HistoryError, Month};

/// The most months one answer holds per metric: ten years.
const MAX_MONTHS: u32 = 120;

/// The run of months a reader asks for, both ends counted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryWindow {
    from: Month,
    to: Month,
}

impl HistoryWindow {
    /// Fills in the ends that were not asked for: the window ends with the
    /// last full month before `now` and starts 120 months before its end.
    /// A window that is longer, or that ends before it starts, is refused,
    /// so the size of an answer is bounded before anything is read.
    pub fn resolve(
        from: Option<Month>,
        to: Option<Month>,
        now: DateTime<Utc>,
    ) -> Result<Self, HistoryError> {
        let to = match to {
            Some(to) => to,
            None => Month::containing(now.date_naive())?.back(1),
        };
        let from = from.unwrap_or_else(|| to.back(MAX_MONTHS - 1));

        let months = from.months_through(to);

        if months < 1 || months > MAX_MONTHS as i32 {
            return Err(HistoryError::BadWindow { max: MAX_MONTHS });
        }

        Ok(Self { from, to })
    }

    pub fn from(&self) -> Month {
        self.from
    }

    pub fn to(&self) -> Month {
        self.to
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn month(raw: &str) -> Month {
        Month::parse(raw).expect("month")
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 9, 12, 0, 0).unwrap()
    }

    #[test]
    fn with_nothing_asked_it_is_the_last_120_full_months() {
        let window = HistoryWindow::resolve(None, None, now()).expect("window");

        assert_eq!(window.from(), month("2016-10"));
        assert_eq!(
            window.to(),
            month("2026-09"),
            "the month still running is not a full month"
        );
    }

    #[test]
    fn an_end_alone_reaches_back_120_months_from_it() {
        let window = HistoryWindow::resolve(None, Some(month("2020-12")), now()).expect("window");

        assert_eq!(window.from(), month("2011-01"));
        assert_eq!(window.to(), month("2020-12"));
    }

    #[test]
    fn a_start_alone_runs_to_the_last_full_month() {
        let window = HistoryWindow::resolve(Some(month("2025-01")), None, now()).expect("window");

        assert_eq!(window.from(), month("2025-01"));
        assert_eq!(window.to(), month("2026-09"));
    }

    #[test]
    fn both_ends_are_kept_as_asked_and_one_month_is_a_window() {
        let window = HistoryWindow::resolve(Some(month("2024-05")), Some(month("2024-05")), now())
            .expect("window");

        assert_eq!(window.from(), window.to());
    }

    #[test]
    fn a_window_that_ends_before_it_starts_is_refused() {
        assert!(matches!(
            HistoryWindow::resolve(Some(month("2024-06")), Some(month("2024-05")), now()),
            Err(HistoryError::BadWindow { .. })
        ));
    }

    #[test]
    fn more_than_120_months_is_refused_rather_than_cut_short() {
        assert!(
            HistoryWindow::resolve(Some(month("2016-10")), Some(month("2026-09")), now()).is_ok()
        );
        assert!(matches!(
            HistoryWindow::resolve(Some(month("2016-09")), Some(month("2026-09")), now()),
            Err(HistoryError::BadWindow { max: 120 })
        ));
        assert!(
            matches!(
                HistoryWindow::resolve(Some(month("2010-01")), None, now()),
                Err(HistoryError::BadWindow { .. })
            ),
            "a start alone that lies too far back is not silently moved"
        );
    }
}
