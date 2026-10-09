use std::sync::Arc;

use tokio::sync::OnceCell;

use crate::features::zones::{
    app::{AppError, ZoneRepository},
    domain::{Place, PlaceIndex},
};

/// Answers which place a point is in: governorate, zone and sub-zone, or
/// nothing when it is outside every sub-zone shape.
///
/// The shapes are read from the repository the first time a point is asked
/// about and kept for as long as the server runs. They are reference data
/// that change only by migration, which restarts the server anyway.
pub struct LocatePlaceUseCase {
    repository: Arc<dyn ZoneRepository>,
    index: OnceCell<PlaceIndex>,
}

impl LocatePlaceUseCase {
    pub fn new(repository: Arc<dyn ZoneRepository>) -> Self {
        Self {
            repository,
            index: OnceCell::new(),
        }
    }

    pub async fn execute(&self, lat: f64, lon: f64) -> Result<Option<Place>, AppError> {
        // A failed read is not kept: the next point asked about tries again.
        let index = self.index.get_or_try_init(|| self.load()).await?;

        Ok(index.locate(lat, lon).cloned())
    }

    async fn load(&self) -> Result<PlaceIndex, AppError> {
        let zones = self.repository.find_all_zones().await?;
        let shapes = self.repository.find_all_sub_zone_shapes().await?;

        let index = PlaceIndex::new(&zones, shapes);

        if index.is_empty() {
            tracing::warn!("no sub-zone has a shape: every point will be outside every place");
        } else {
            tracing::info!(shapes = index.len(), "sub-zone shapes loaded");
        }

        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::zones::app::testing::{FakeZoneRepository, RepositoryCall, shapes};

    fn use_case(repository: &FakeZoneRepository) -> LocatePlaceUseCase {
        LocatePlaceUseCase::new(Arc::new(repository.clone()))
    }

    #[tokio::test]
    async fn a_point_inside_a_sub_zone_gets_its_governorate_zone_and_sub_zone() {
        let repository = FakeZoneRepository::seeded().with_shapes(shapes());

        let place = use_case(&repository)
            .execute(35.5, 46.5)
            .await
            .expect("answer")
            .expect("place");

        assert_eq!(place.governorate(), "Sulaymaniyah");
        assert_eq!(place.zone_slug().as_str(), "chamchamal");
        assert_eq!(place.sub_zone_slug().as_str(), "sangaw");
    }

    #[tokio::test]
    async fn a_point_outside_every_shape_has_no_place() {
        let repository = FakeZoneRepository::seeded().with_shapes(shapes());

        assert_eq!(
            use_case(&repository)
                .execute(38.5, 44.5)
                .await
                .expect("answer"),
            None
        );
    }

    #[tokio::test]
    async fn the_shapes_are_read_once_however_many_points_are_asked_about() {
        let repository = FakeZoneRepository::seeded().with_shapes(shapes());
        let use_case = use_case(&repository);

        for _ in 0..3 {
            use_case.execute(35.5, 44.5).await.expect("answer");
        }

        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindAllZones,
                RepositoryCall::FindAllSubZoneShapes
            ]
        );
    }

    #[tokio::test]
    async fn a_failed_read_surfaces_and_the_next_point_tries_again() {
        let repository = FakeZoneRepository::seeded().with_shapes(shapes());
        let use_case = use_case(&repository);

        repository.fail(true);
        assert!(use_case.execute(35.5, 44.5).await.is_err());

        repository.fail(false);
        assert!(
            use_case
                .execute(35.5, 44.5)
                .await
                .expect("answer")
                .is_some(),
            "a failure must not be remembered as an empty map"
        );
    }

    #[tokio::test]
    async fn with_no_shapes_stored_every_point_is_outside() {
        let repository = FakeZoneRepository::seeded();

        assert_eq!(
            use_case(&repository)
                .execute(35.5, 44.5)
                .await
                .expect("answer"),
            None
        );
    }
}
