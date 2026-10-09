use std::sync::Arc;

use super::{RecordSubZoneReadingInput, locate::sub_zone_named};
use crate::{
    app::StaffContext,
    features::zones::{
        app::{AppError, ZoneRepository},
        domain::SubZoneReading,
    },
};

pub struct CreateSubZoneReadingUseCase {
    repository: Arc<dyn ZoneRepository>,
}

impl CreateSubZoneReadingUseCase {
    pub fn new(repository: Arc<dyn ZoneRepository>) -> Self {
        Self { repository }
    }

    /// Stores a dryness a staff member typed in for a month the sub-zone
    /// has none for. The insert itself decides whether there is one.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        input: RecordSubZoneReadingInput,
    ) -> Result<SubZoneReading, AppError> {
        let sub_zone = sub_zone_named(
            self.repository.as_ref(),
            &input.zone_slug,
            &input.sub_zone_slug,
        )
        .await?;

        let reading = SubZoneReading::new(*sub_zone.id(), input.month, input.dryness);

        let Some(stored) = self.repository.insert_sub_zone_reading(&reading).await? else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                zone_slug = input.zone_slug.as_str(),
                sub_zone_slug = input.sub_zone_slug.as_str(),
                month = %String::from(input.month),
                "sub-zone reading not created: one is already stored"
            );

            return Err(AppError::ReadingAlreadyExists);
        };

        tracing::info!(
            staff_id = *actor.staff_id(),
            zone_slug = input.zone_slug.as_str(),
            sub_zone_slug = input.sub_zone_slug.as_str(),
            month = %String::from(input.month),
            dryness = stored.dryness().value(),
            "sub-zone reading created from the dashboard"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::AppError as GlobalAppError,
        features::zones::app::testing::{
            FakeZoneRepository, RepositoryCall, a_dryness, a_month, a_slug, a_sub_zone_reading,
            an_actor,
        },
    };

    fn input(zone: &str, sub_zone: &str) -> RecordSubZoneReadingInput {
        RecordSubZoneReadingInput {
            zone_slug: a_slug(zone),
            sub_zone_slug: a_slug(sub_zone),
            month: a_month("2026-03"),
            dryness: a_dryness(64),
        }
    }

    #[tokio::test]
    async fn stores_a_reading_for_a_month_the_sub_zone_has_none_for() {
        let repository = FakeZoneRepository::seeded();
        let use_case = CreateSubZoneReadingUseCase::new(Arc::new(repository.clone()));

        let stored = use_case
            .execute(&an_actor(7), input("chamchamal", "sangaw"))
            .await
            .expect("created");

        assert_eq!(*stored.sub_zone_id(), 3);
        assert_eq!(repository.stored_sub_zone_readings(), vec![stored]);
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindZoneBySlug {
                    slug: "chamchamal".to_string(),
                },
                RepositoryCall::FindSubZoneBySlug {
                    zone_id: 1,
                    slug: "sangaw".to_string(),
                },
                RepositoryCall::InsertSubZoneReading {
                    sub_zone_id: 3,
                    month: "2026-03".to_string(),
                },
            ]
        );
    }

    #[tokio::test]
    async fn a_month_that_already_has_a_reading_is_refused_and_left_as_it_was() {
        let repository = FakeZoneRepository::seeded()
            .with_sub_zone_readings(vec![a_sub_zone_reading(3, "2026-03", 20)]);
        let use_case = CreateSubZoneReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&an_actor(7), input("chamchamal", "sangaw"))
            .await;

        assert!(matches!(result, Err(AppError::ReadingAlreadyExists)));
        assert_eq!(
            repository.stored_sub_zone_readings()[0].dryness().value(),
            20
        );
    }

    #[tokio::test]
    async fn a_sub_zone_of_another_zone_is_not_found_and_nothing_is_written() {
        let repository = FakeZoneRepository::seeded();
        let use_case = CreateSubZoneReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&an_actor(7), input("kalar", "sangaw"))
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(!repository.wrote());
    }
}
