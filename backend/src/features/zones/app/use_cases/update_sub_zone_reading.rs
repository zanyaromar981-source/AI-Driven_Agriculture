use std::sync::Arc;

use super::{RecordSubZoneReadingInput, locate::sub_zone_named};
use crate::{
    app::{AppError as GlobalAppError, StaffContext},
    features::zones::{
        app::{AppError, ZoneRepository},
        domain::SubZoneReading,
    },
};

pub struct UpdateSubZoneReadingUseCase {
    repository: Arc<dyn ZoneRepository>,
}

impl UpdateSubZoneReadingUseCase {
    pub fn new(repository: Arc<dyn ZoneRepository>) -> Self {
        Self { repository }
    }

    /// Replaces the dryness stored for that sub-zone and month. The update
    /// itself decides whether there is one.
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

        let Some(stored) = self.repository.update_sub_zone_reading(&reading).await? else {
            return Err(GlobalAppError::NotFound.into());
        };

        tracing::info!(
            staff_id = *actor.staff_id(),
            zone_slug = input.zone_slug.as_str(),
            sub_zone_slug = input.sub_zone_slug.as_str(),
            month = %String::from(input.month),
            dryness = stored.dryness().value(),
            "sub-zone reading updated from the dashboard"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::zones::app::testing::{
        FakeZoneRepository, RepositoryCall, a_dryness, a_month, a_slug, a_sub_zone_reading,
        an_actor,
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
    async fn replaces_the_stored_reading_of_that_sub_zone_and_month_only() {
        let repository = FakeZoneRepository::seeded().with_sub_zone_readings(vec![
            a_sub_zone_reading(3, "2026-03", 20),
            a_sub_zone_reading(3, "2026-02", 21),
            a_sub_zone_reading(2, "2026-03", 22),
        ]);
        let use_case = UpdateSubZoneReadingUseCase::new(Arc::new(repository.clone()));

        let stored = use_case
            .execute(&an_actor(7), input("chamchamal", "sangaw"))
            .await
            .expect("updated");

        assert_eq!(stored.dryness().value(), 64);
        assert_eq!(
            repository
                .stored_sub_zone_readings()
                .iter()
                .map(|reading| reading.dryness().value())
                .collect::<Vec<_>>(),
            vec![64, 21, 22]
        );
        assert_eq!(
            repository.calls().last(),
            Some(&RepositoryCall::UpdateSubZoneReading {
                sub_zone_id: 3,
                month: "2026-03".to_string(),
            })
        );
    }

    #[tokio::test]
    async fn a_month_with_no_reading_is_not_found_and_none_is_created() {
        let repository = FakeZoneRepository::seeded();
        let use_case = UpdateSubZoneReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&an_actor(7), input("chamchamal", "sangaw"))
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(repository.stored_sub_zone_readings().is_empty());
    }

    #[tokio::test]
    async fn a_sub_zone_of_another_zone_is_not_found_and_nothing_is_written() {
        let repository = FakeZoneRepository::seeded();
        let use_case = UpdateSubZoneReadingUseCase::new(Arc::new(repository.clone()));

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
