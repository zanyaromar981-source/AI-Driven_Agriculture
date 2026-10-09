use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        alwa::{
            app::{AppError, ZoneLocator},
            domain::{GeoPoint, ZoneSlug},
        },
        zones::app::use_cases::LocatePlaceUseCase,
    },
};

/// Finds the district of a point by asking the zones feature, which keeps
/// the sub-district shapes in memory once it has read them.
pub struct ZonesFeatureZoneLocator {
    zones: Arc<LocatePlaceUseCase>,
}

impl ZonesFeatureZoneLocator {
    pub fn new(zones: Arc<LocatePlaceUseCase>) -> Self {
        Self { zones }
    }
}

impl std::fmt::Debug for ZonesFeatureZoneLocator {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("ZonesFeatureZoneLocator").finish()
    }
}

#[async_trait]
impl ZoneLocator for ZonesFeatureZoneLocator {
    async fn zone_of(&self, point: &GeoPoint) -> Result<Option<ZoneSlug>, AppError> {
        let place = self
            .zones
            .execute(point.lat(), point.lon())
            .await
            .map_err(|error| {
                tracing::error!(%error, "looking up the zone of a listing failed");

                AppError::from(GlobalAppError::InternalServerError)
            })?;

        place
            .map(|place| ZoneSlug::new(String::from(place.zone_slug())))
            .transpose()
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::zones::app::testing::{FakeZoneRepository, shapes};

    fn locator(zones: &FakeZoneRepository) -> ZonesFeatureZoneLocator {
        ZonesFeatureZoneLocator::new(Arc::new(LocatePlaceUseCase::new(Arc::new(zones.clone()))))
    }

    fn point(lat: f64, lon: f64) -> GeoPoint {
        GeoPoint::in_region(lat, lon).expect("point")
    }

    #[tokio::test]
    async fn a_point_inside_a_district_gets_its_zone_slug() {
        let zones = FakeZoneRepository::seeded().with_shapes(shapes());

        let zone = locator(&zones)
            .zone_of(&point(34.5, 44.5))
            .await
            .expect("answer")
            .expect("zone");

        assert_eq!(zone.as_str(), "kalar");
    }

    #[tokio::test]
    async fn a_point_outside_every_district_has_no_zone() {
        let zones = FakeZoneRepository::seeded().with_shapes(shapes());

        assert_eq!(
            locator(&zones)
                .zone_of(&point(38.5, 44.5))
                .await
                .expect("answer"),
            None
        );
    }

    #[tokio::test]
    async fn a_failure_in_the_zones_feature_is_a_server_fault_not_a_missing_zone() {
        let result = locator(&FakeZoneRepository::failing())
            .zone_of(&point(34.5, 44.5))
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(
                GlobalAppError::InternalServerError
            ))
        ));
    }
}
