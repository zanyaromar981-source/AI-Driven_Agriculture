use chrono::{DateTime, Utc};
use getset::CopyGetters;

use crate::{features::farms::domain::FarmError, shared::DomainError};

/// A box around the Kurdistan Region with a margin. It sits inside UTM zone
/// 38N (42 to 48 degrees east), so every accepted point projects cleanly,
/// and it keeps a mistyped corner from stretching an outline across a
/// continent.
const MIN_LON: f64 = 41.0;
const MAX_LON: f64 = 48.0;
const MIN_LAT: f64 = 33.0;
const MAX_LAT: f64 = 39.0;

/// A GPS corner the farmer tapped while walking the field edge.
#[derive(Clone, Copy, Debug, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Point {
    lat: f64,
    lon: f64,
    accuracy_m: Option<f64>,
    recorded_at: Option<DateTime<Utc>>,
}

impl Point {
    pub fn new(
        lat: f64,
        lon: f64,
        accuracy_m: Option<f64>,
        recorded_at: Option<DateTime<Utc>>,
    ) -> Result<Self, FarmError> {
        if !(MIN_LAT..=MAX_LAT).contains(&lat) || !(MIN_LON..=MAX_LON).contains(&lon) {
            return Err(DomainError::InvalidValue(
                "Point is outside the area the farm grid covers".to_string(),
            )
            .into());
        }

        if accuracy_m.is_some_and(|accuracy| !accuracy.is_finite() || accuracy < 0.0) {
            return Err(DomainError::InvalidValue(
                "Point accuracy must be zero or more metres".to_string(),
            )
            .into());
        }

        Ok(Self {
            lat,
            lon,
            accuracy_m,
            recorded_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_point_in_kurdistan() {
        assert!(Point::new(36.0312, 44.6021, Some(6.0), None).is_ok());
    }

    #[test]
    fn rejects_a_point_far_outside_the_grid_zone() {
        assert!(Point::new(36.0, 10.0, None, None).is_err());
        assert!(Point::new(-36.0, 44.6, None, None).is_err());
        assert!(Point::new(30.0, 44.6, None, None).is_err());
        assert!(Point::new(36.0, 50.0, None, None).is_err());
    }

    #[test]
    fn rejects_swapped_latitude_and_longitude() {
        assert!(
            Point::new(44.6021, 36.0312, None, None).is_err(),
            "a swapped pair must not silently land in another country"
        );
    }

    #[test]
    fn rejects_values_that_are_not_numbers() {
        assert!(Point::new(f64::NAN, 44.6, None, None).is_err());
        assert!(Point::new(36.0, f64::INFINITY, None, None).is_err());
    }

    #[test]
    fn rejects_a_negative_accuracy() {
        assert!(Point::new(36.0, 44.6, Some(-1.0), None).is_err());
    }
}
