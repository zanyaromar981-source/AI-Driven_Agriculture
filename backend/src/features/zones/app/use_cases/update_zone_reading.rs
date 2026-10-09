use std::sync::Arc;

use super::{RecordZoneReadingInput, locate::zone_named};
use crate::{
    app::{AppError as GlobalAppError, StaffContext},
    features::zones::{
        app::{AppError, ZoneRepository},
        domain::ZoneReading,
    },
};

pub struct UpdateZoneReadingUseCase {
    repository: Arc<dyn ZoneRepository>,
}

impl UpdateZoneReadingUseCase {
    pub fn new(repository: Arc<dyn ZoneRepository>) -> Self {
        Self { repository }
    }

    /// Replaces every measured field of the reading stored for that zone
    /// and month. Whether there is one is decided by the update itself, so
    /// a reading deleted at the same moment is not brought back.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        input: RecordZoneReadingInput,
    ) -> Result<ZoneReading, AppError> {
        let zone = zone_named(self.repository.as_ref(), &input.zone_slug).await?;

        let reading = ZoneReading::new(
            *zone.id(),
            input.month,
            input.dryness,
            input.rain_pct_of_normal,
            input.greenness_pct_vs_normal,
            input.water_need,
            input.nitrogen_hold,
            input.best_crops,
            input.source,
        )?;

        let Some(stored) = self.repository.update_reading(&reading).await? else {
            return Err(GlobalAppError::NotFound.into());
        };

        tracing::info!(
            staff_id = *actor.staff_id(),
            zone_slug = input.zone_slug.as_str(),
            month = %String::from(input.month),
            dryness = stored.dryness().value(),
            source = stored.source().as_str(),
            "zone reading updated from the dashboard"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::zones::{
        app::testing::{
            FakeZoneRepository, RepositoryCall, a_dryness, a_month, a_reading, a_slug, a_source,
            an_actor,
        },
        domain::{Crop, WaterNeed, ZoneError},
    };

    fn input(zone_slug: &str, best_crops: Vec<Crop>) -> RecordZoneReadingInput {
        RecordZoneReadingInput {
            zone_slug: a_slug(zone_slug),
            month: a_month("2026-03"),
            dryness: a_dryness(72),
            rain_pct_of_normal: None,
            greenness_pct_vs_normal: None,
            water_need: Some(WaterNeed::new(80).expect("need")),
            nitrogen_hold: true,
            best_crops,
            source: a_source(),
        }
    }

    #[tokio::test]
    async fn replaces_the_stored_reading_of_that_zone_and_month_only() {
        let repository = FakeZoneRepository::seeded().with_readings(vec![
            a_reading(2, "2026-03", 40),
            a_reading(2, "2026-02", 41),
            a_reading(1, "2026-03", 42),
        ]);
        let use_case = UpdateZoneReadingUseCase::new(Arc::new(repository.clone()));

        let stored = use_case
            .execute(&an_actor(7), input("kalar", vec![Crop::Barley]))
            .await
            .expect("updated");

        assert_eq!(stored.dryness().value(), 72);
        assert_eq!(stored.water_need().map(|need| need.value()), Some(80));
        assert_eq!(
            repository
                .stored_readings()
                .iter()
                .map(|reading| reading.dryness().value())
                .collect::<Vec<_>>(),
            vec![72, 41, 42]
        );
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindZoneBySlug {
                    slug: "kalar".to_string(),
                },
                RepositoryCall::UpdateReading {
                    zone_id: 2,
                    month: "2026-03".to_string(),
                },
            ],
            "the update decides: there is no lookup of the reading first"
        );
    }

    #[tokio::test]
    async fn a_month_with_no_reading_is_not_found_and_none_is_created() {
        let repository = FakeZoneRepository::seeded();
        let use_case = UpdateZoneReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&an_actor(7), input("kalar", vec![])).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(
            repository.stored_readings().is_empty(),
            "an update must never create"
        );
    }

    #[tokio::test]
    async fn an_unknown_zone_is_not_found_and_nothing_is_written() {
        let repository = FakeZoneRepository::seeded();
        let use_case = UpdateZoneReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&an_actor(7), input("atlantis", vec![]))
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn a_crop_ranked_twice_is_refused_and_nothing_is_written() {
        let repository =
            FakeZoneRepository::seeded().with_readings(vec![a_reading(2, "2026-03", 40)]);
        let use_case = UpdateZoneReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&an_actor(7), input("kalar", vec![Crop::Wheat, Crop::Wheat]))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Zone(ZoneError::RepeatedCrop(_)))
        ));
        assert!(!repository.wrote());
    }
}
