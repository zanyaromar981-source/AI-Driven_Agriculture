use getset::CopyGetters;

use crate::features::fires::domain::FireError;

/// A box around the Kurdistan Region with room to spare. A detection outside
/// it is a job pointed at the wrong area or a swapped pair.
const MIN_LAT: f64 = 28.0;
const MAX_LAT: f64 = 40.0;
const MIN_LON: f64 = 38.0;
const MAX_LON: f64 = 50.0;

/// Where the satellite saw the fire, WGS84 decimal degrees.
#[derive(Clone, Copy, Debug, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct FireLocation {
    lat: f64,
    lon: f64,
}

impl FireLocation {
    pub fn new(lat: f64, lon: f64) -> Result<Self, FireError> {
        if !(MIN_LAT..=MAX_LAT).contains(&lat) || !(MIN_LON..=MAX_LON).contains(&lon) {
            return Err(FireError::OutsideRegion);
        }

        Ok(Self { lat, lon })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_point_in_kurdistan() {
        assert!(FireLocation::new(35.5312, 44.8321).is_ok());
    }

    #[test]
    fn the_edges_of_the_box_are_inside() {
        assert!(FireLocation::new(28.0, 38.0).is_ok());
        assert!(FireLocation::new(40.0, 50.0).is_ok());
    }

    #[test]
    fn rejects_a_point_outside_the_box() {
        assert!(matches!(
            FireLocation::new(27.99, 44.0),
            Err(FireError::OutsideRegion)
        ));
        assert!(FireLocation::new(40.01, 44.0).is_err());
        assert!(FireLocation::new(36.0, 37.99).is_err());
        assert!(FireLocation::new(36.0, 50.01).is_err());
    }

    #[test]
    fn rejects_values_that_are_not_numbers() {
        assert!(FireLocation::new(f64::NAN, 44.0).is_err());
        assert!(FireLocation::new(36.0, f64::INFINITY).is_err());
    }
}
