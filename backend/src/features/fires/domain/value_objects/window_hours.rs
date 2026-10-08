use crate::features::fires::domain::FireError;

const MIN_HOURS: i64 = 1;
const MAX_HOURS: i64 = 168;
const DEFAULT_HOURS: i64 = 24;

/// How far back the dashboard looks for fires: one hour to one week.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowHours(i64);

impl WindowHours {
    pub fn new(value: i64) -> Result<Self, FireError> {
        if !(MIN_HOURS..=MAX_HOURS).contains(&value) {
            return Err(FireError::WindowOutOfRange {
                min: MIN_HOURS,
                max: MAX_HOURS,
            });
        }

        Ok(Self(value))
    }

    pub fn hours(&self) -> i64 {
        self.0
    }
}

impl Default for WindowHours {
    fn default() -> Self {
        Self(DEFAULT_HOURS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_window_is_one_day() {
        assert_eq!(WindowHours::default().hours(), 24);
    }

    #[test]
    fn one_hour_to_one_week_is_accepted() {
        assert!(WindowHours::new(1).is_ok());
        assert!(WindowHours::new(168).is_ok());
    }

    #[test]
    fn anything_outside_is_refused_rather_than_clamped() {
        assert!(matches!(
            WindowHours::new(0),
            Err(FireError::WindowOutOfRange { min: 1, max: 168 })
        ));
        assert!(WindowHours::new(169).is_err());
        assert!(WindowHours::new(-24).is_err());
    }
}
