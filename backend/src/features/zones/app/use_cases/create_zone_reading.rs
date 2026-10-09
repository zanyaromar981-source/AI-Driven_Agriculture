use std::sync::Arc;

use super::{RecordZoneReadingInput, locate::zone_named};
use crate::{
    app::StaffContext,
    features::zones::{
        app::{AppError, ZoneRepository},
        domain::ZoneReading,
    },
};

pub struct CreateZoneReadingUseCase {
    repository: Arc<dyn ZoneRepository>,
}

impl CreateZoneReadingUseCase {
    pub fn new(repository: Arc<dyn ZoneRepository>) -> Self {
        Self { repository }
    }

    /// Stores a reading a staff member typed in for a month the zone has
    /// none for. Whether there is one already is decided by the insert
    /// itself, so two creates sent at the same moment store exactly one.
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

        let Some(stored) = self.repository.insert_reading(&reading).await? else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                zone_slug = input.zone_slug.as_str(),
                month = %String::from(input.month),
                "zone reading not created: one is already stored"
            );

            return Err(AppError::ReadingAlreadyExists);
        };

        tracing::info!(
            staff_id = *actor.staff_id(),
            zone_slug = input.zone_slug.as_str(),
            month = %String::from(input.month),
            dryness = stored.dryness().value(),
            source = stored.source().as_str(),
            "zone reading created from the dashboard"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::AppError as GlobalAppError,
        features::zones::{
            app::testing::{
                FakeZoneRepository, RepositoryCall, a_dryness, a_month, a_reading, a_slug,
                a_source, an_actor,
            },
            domain::{Crop, ZoneError},
        },
    };

    fn input(zone_slug: &str, dryness: i32, best_crops: Vec<Crop>) -> RecordZoneReadingInput {
        RecordZoneReadingInput {
            zone_slug: a_slug(zone_slug),
            month: a_month("2026-03"),
            dryness: a_dryness(dryness),
            rain_pct_of_normal: None,
            greenness_pct_vs_normal: None,
            water_need: None,
            nitrogen_hold: false,
            best_crops,
            source: a_source(),
        }
    }

    #[tokio::test]
    async fn stores_a_reading_for_a_month_the_zone_has_none_for() {
        let repository = FakeZoneRepository::seeded();
        let use_case = CreateZoneReadingUseCase::new(Arc::new(repository.clone()));

        let stored = use_case
            .execute(&an_actor(7), input("kalar", 72, vec![Crop::Barley]))
            .await
            .expect("created");

        assert!(stored.id().is_some(), "the stored reading is returned");
        assert_eq!(*stored.zone_id(), 2);
        assert_eq!(repository.stored_readings(), vec![stored]);
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindZoneBySlug {
                    slug: "kalar".to_string(),
                },
                RepositoryCall::InsertReading {
                    zone_id: 2,
                    month: "2026-03".to_string(),
                },
            ],
            "the insert decides: there is no lookup of the reading first"
        );
    }

    #[tokio::test]
    async fn a_month_that_already_has_a_reading_is_refused_and_left_as_it_was() {
        let repository =
            FakeZoneRepository::seeded().with_readings(vec![a_reading(2, "2026-03", 40)]);
        let use_case = CreateZoneReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&an_actor(7), input("kalar", 72, vec![]))
            .await;

        assert!(matches!(result, Err(AppError::ReadingAlreadyExists)));
        assert_eq!(
            repository.stored_readings()[0].dryness().value(),
            40,
            "a create must never replace what is stored"
        );
    }

    #[tokio::test]
    async fn the_same_month_of_another_zone_is_not_in_the_way() {
        let repository =
            FakeZoneRepository::seeded().with_readings(vec![a_reading(1, "2026-03", 40)]);
        let use_case = CreateZoneReadingUseCase::new(Arc::new(repository.clone()));

        assert!(
            use_case
                .execute(&an_actor(7), input("kalar", 72, vec![]))
                .await
                .is_ok()
        );
        assert_eq!(repository.stored_readings().len(), 2);
    }

    #[tokio::test]
    async fn an_unknown_zone_is_not_found_and_nothing_is_written() {
        let repository = FakeZoneRepository::seeded();
        let use_case = CreateZoneReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&an_actor(7), input("atlantis", 72, vec![]))
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn a_crop_ranked_twice_is_refused_as_the_ingest_route_refuses_it() {
        let repository = FakeZoneRepository::seeded();
        let use_case = CreateZoneReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(
                &an_actor(7),
                input("kalar", 72, vec![Crop::Wheat, Crop::Wheat]),
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Zone(ZoneError::RepeatedCrop(_)))
        ));
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = CreateZoneReadingUseCase::new(Arc::new(FakeZoneRepository::failing()));

        assert!(
            use_case
                .execute(&an_actor(7), input("kalar", 72, vec![]))
                .await
                .is_err()
        );
    }
}
