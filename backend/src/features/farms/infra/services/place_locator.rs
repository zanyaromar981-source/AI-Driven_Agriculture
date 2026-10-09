use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        farms::{
            app::{AppError, PlaceLocator},
            domain::FarmPlace,
        },
        zones::app::use_cases::LocatePlaceUseCase,
    },
};

/// Finds the place of a point by asking the zones feature, which keeps the
/// sub-district shapes in memory once it has read them.
pub struct ZonesFeaturePlaceLocator {
    zones: Arc<LocatePlaceUseCase>,
}

impl ZonesFeaturePlaceLocator {
    pub fn new(zones: Arc<LocatePlaceUseCase>) -> Self {
        Self { zones }
    }
}

impl std::fmt::Debug for ZonesFeaturePlaceLocator {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("ZonesFeaturePlaceLocator").finish()
    }
}

#[async_trait]
impl PlaceLocator for ZonesFeaturePlaceLocator {
    async fn locate(&self, lat: f64, lon: f64) -> Result<Option<FarmPlace>, AppError> {
        let place = self.zones.execute(lat, lon).await.map_err(|error| {
            tracing::error!(%error, "looking up the place of a farm failed");

            AppError::from(GlobalAppError::InternalServerError)
        })?;

        place
            .map(|place| {
                FarmPlace::new(
                    place.governorate().clone(),
                    String::from(place.zone_slug()),
                    String::from(place.sub_zone_slug()),
                )
            })
            .transpose()
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::zones::app::testing::{FakeZoneRepository, RepositoryCall, shapes};

    fn locator(zones: &FakeZoneRepository) -> ZonesFeaturePlaceLocator {
        ZonesFeaturePlaceLocator::new(Arc::new(LocatePlaceUseCase::new(Arc::new(zones.clone()))))
    }

    #[tokio::test]
    async fn a_point_inside_a_sub_zone_gets_the_three_parts_of_its_place() {
        let zones = FakeZoneRepository::seeded().with_shapes(shapes());

        let place = locator(&zones)
            .locate(34.5, 44.5)
            .await
            .expect("answer")
            .expect("place");

        assert_eq!(place.governorate(), "Sulaymaniyah");
        assert_eq!(place.zone_slug(), "kalar");
        assert_eq!(place.sub_zone_slug(), "markaz-kalar");
    }

    #[tokio::test]
    async fn a_point_outside_every_sub_zone_has_no_place() {
        let zones = FakeZoneRepository::seeded().with_shapes(shapes());

        assert_eq!(
            locator(&zones).locate(38.5, 44.5).await.expect("answer"),
            None
        );
    }

    #[tokio::test]
    async fn the_shapes_are_read_once_for_any_number_of_farms() {
        let zones = FakeZoneRepository::seeded().with_shapes(shapes());
        let locator = locator(&zones);

        for _ in 0..5 {
            locator.locate(34.5, 44.5).await.expect("answer");
        }

        assert_eq!(
            zones
                .calls()
                .iter()
                .filter(|call| **call == RepositoryCall::FindAllSubZoneShapes)
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn a_failure_in_the_zones_feature_is_a_server_fault_not_a_missing_place() {
        let result = locator(&FakeZoneRepository::failing())
            .locate(34.5, 44.5)
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(
                GlobalAppError::InternalServerError
            ))
        ));
    }
}
