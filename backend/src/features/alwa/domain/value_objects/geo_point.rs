use getset::CopyGetters;

use crate::{features::alwa::domain::AlwaError, shared::DomainError};

/// The same box around the Kurdistan Region the farm grid uses. A crop on
/// sale and an alwa both stand inside it; a point outside is a mistyped or
/// swapped pair.
const MIN_LON: f64 = 41.0;
const MAX_LON: f64 = 48.0;
const MIN_LAT: f64 = 33.0;
const MAX_LAT: f64 = 39.0;

const EARTH_RADIUS_KM: f64 = 6371.0;

/// A place on the map, WGS84.
#[derive(Clone, Copy, Debug, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct GeoPoint {
    lat: f64,
    lon: f64,
}

impl GeoPoint {
    /// Where a crop or an alwa is: inside the region.
    pub fn in_region(lat: f64, lon: f64) -> Result<Self, AlwaError> {
        if !(MIN_LAT..=MAX_LAT).contains(&lat) || !(MIN_LON..=MAX_LON).contains(&lon) {
            return Err(DomainError::InvalidValue(
                "The point is outside the Kurdistan Region".to_string(),
            )
            .into());
        }

        Ok(Self { lat, lon })
    }

    /// Where a reader of the board stands. A buyer may look from anywhere,
    /// so only the range of the numbers is checked.
    pub fn anywhere(lat: f64, lon: f64) -> Result<Self, AlwaError> {
        if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
            return Err(DomainError::InvalidValue(
                "Latitude must be -90 to 90 and longitude -180 to 180".to_string(),
            )
            .into());
        }

        Ok(Self { lat, lon })
    }

    /// A point comes as two numbers that may each be missing: both make a
    /// point, neither makes none, and one alone is a mistake.
    pub fn from_pair(
        lat: Option<f64>,
        lon: Option<f64>,
        build: fn(f64, f64) -> Result<Self, AlwaError>,
    ) -> Result<Option<Self>, AlwaError> {
        match (lat, lon) {
            (Some(lat), Some(lon)) => build(lat, lon).map(Some),
            (None, None) => Ok(None),
            _ => Err(
                DomainError::InvalidValue("lat and lon must be given together".to_string()).into(),
            ),
        }
    }

    /// The distance along the surface of the earth (haversine), in
    /// kilometres. The board is put in this order by the database with the
    /// same formula and the same radius.
    pub fn km_to(&self, other: &GeoPoint) -> f64 {
        let half_lat = (other.lat - self.lat).to_radians() / 2.0;
        let half_lon = (other.lon - self.lon).to_radians() / 2.0;

        let a = half_lat.sin().powi(2)
            + self.lat.to_radians().cos() * other.lat.to_radians().cos() * half_lon.sin().powi(2);

        2.0 * EARTH_RADIUS_KM * a.sqrt().min(1.0).asin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_point_in_the_region_is_accepted_for_a_listing() {
        let point = GeoPoint::in_region(35.5572, 45.4356).expect("point");

        assert_eq!(point.lat(), 35.5572);
        assert_eq!(point.lon(), 45.4356);
    }

    #[test]
    fn a_listing_point_outside_the_region_or_swapped_is_refused() {
        assert!(
            GeoPoint::in_region(33.3, 44.4).is_ok(),
            "the box has a margin"
        );
        assert!(GeoPoint::in_region(30.0, 44.6).is_err());
        assert!(GeoPoint::in_region(36.0, 50.0).is_err());
        assert!(GeoPoint::in_region(45.4356, 35.5572).is_err());
        assert!(GeoPoint::in_region(f64::NAN, 45.0).is_err());
    }

    #[test]
    fn a_reader_may_stand_anywhere_on_earth_but_not_off_it() {
        assert!(GeoPoint::anywhere(51.5, -0.12).is_ok());
        assert!(GeoPoint::anywhere(91.0, 0.0).is_err());
        assert!(GeoPoint::anywhere(0.0, 181.0).is_err());
        assert!(GeoPoint::anywhere(f64::NAN, 0.0).is_err());
    }

    #[test]
    fn a_point_needs_both_numbers_or_neither() {
        assert!(matches!(
            GeoPoint::from_pair(Some(35.5), Some(45.4), GeoPoint::in_region),
            Ok(Some(_))
        ));
        assert!(matches!(
            GeoPoint::from_pair(None, None, GeoPoint::in_region),
            Ok(None)
        ));
        assert!(GeoPoint::from_pair(Some(35.5), None, GeoPoint::in_region).is_err());
        assert!(GeoPoint::from_pair(None, Some(45.4), GeoPoint::in_region).is_err());
        assert!(GeoPoint::from_pair(Some(10.0), Some(45.4), GeoPoint::in_region).is_err());
    }

    #[test]
    fn the_distance_between_two_towns_is_what_a_map_says() {
        let sulaymaniyah = GeoPoint::in_region(35.5572, 45.4356).expect("point");
        let erbil = GeoPoint::in_region(36.1911, 44.0092).expect("point");

        let km = sulaymaniyah.km_to(&erbil);

        assert!((145.0..150.0).contains(&km), "got {km}");
        assert_eq!(sulaymaniyah.km_to(&sulaymaniyah), 0.0);
        assert!((erbil.km_to(&sulaymaniyah) - km).abs() < 1e-9);
    }
}
