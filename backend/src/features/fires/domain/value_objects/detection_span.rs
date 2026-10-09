use chrono::{DateTime, Duration, Utc};
use getset::CopyGetters;

use crate::{features::fires::domain::FireError, shared::DomainError};

const DEFAULT_DAYS: i64 = 30;

/// The stretch of detection times a list of stored fires covers. Both ends
/// are inside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct DetectionSpan {
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

impl DetectionSpan {
    /// An end that is left out is filled in: the span ends at `now`, and it
    /// starts 30 days before its end.
    pub fn new(
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Result<Self, FireError> {
        let to = to.unwrap_or(now);
        let from = from.unwrap_or(to - Duration::days(DEFAULT_DAYS));

        if from > to {
            return Err(
                DomainError::InvalidValue("`from` must not be after `to`".to_string()).into(),
            );
        }

        Ok(Self { from, to })
    }

    pub fn contains(&self, moment: DateTime<Utc>) -> bool {
        self.from <= moment && moment <= self.to
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_neither_end_given_it_is_the_last_thirty_days() {
        let now = Utc::now();

        let span = DetectionSpan::new(None, None, now).expect("span");

        assert_eq!(span.to(), now);
        assert_eq!(span.from(), now - Duration::days(30));
    }

    #[test]
    fn with_only_the_end_given_it_starts_thirty_days_before_that_end() {
        let now = Utc::now();
        let to = now - Duration::days(100);

        let span = DetectionSpan::new(None, Some(to), now).expect("span");

        assert_eq!(span.from(), to - Duration::days(30));
        assert_eq!(span.to(), to);
    }

    #[test]
    fn with_only_the_start_given_it_runs_until_now() {
        let now = Utc::now();
        let from = now - Duration::days(400);

        let span = DetectionSpan::new(Some(from), None, now).expect("span");

        assert_eq!(span.from(), from);
        assert_eq!(span.to(), now);
    }

    #[test]
    fn a_start_after_the_end_is_refused() {
        let now = Utc::now();

        assert!(DetectionSpan::new(Some(now), Some(now - Duration::seconds(1)), now).is_err());
        assert!(
            DetectionSpan::new(Some(now + Duration::days(1)), None, now).is_err(),
            "a start in the future with no end is after the filled-in end"
        );
    }

    #[test]
    fn both_ends_are_inside_the_span() {
        let now = Utc::now();
        let from = now - Duration::hours(2);
        let span = DetectionSpan::new(Some(from), Some(now), now).expect("span");

        assert!(span.contains(from));
        assert!(span.contains(now));
        assert!(!span.contains(from - Duration::seconds(1)));
        assert!(!span.contains(now + Duration::seconds(1)));
    }

    #[test]
    fn a_single_moment_is_a_valid_span() {
        let now = Utc::now();

        assert!(DetectionSpan::new(Some(now), Some(now), now).is_ok());
    }
}
