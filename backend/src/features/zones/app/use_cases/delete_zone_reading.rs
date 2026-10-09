use std::sync::Arc;

use super::locate::zone_named;
use crate::{
    app::StaffContext,
    features::zones::{
        app::{AppError, ZoneRepository},
        domain::{Month, ZoneSlug},
    },
};

pub struct DeleteZoneReadingUseCase {
    repository: Arc<dyn ZoneRepository>,
}

impl DeleteZoneReadingUseCase {
    pub fn new(repository: Arc<dyn ZoneRepository>) -> Self {
        Self { repository }
    }

    /// Removes the zone's reading for the month. Deleting what is already
    /// gone succeeds, so a repeated request gets the same answer.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        zone_slug: &ZoneSlug,
        month: Month,
    ) -> Result<(), AppError> {
        let zone = zone_named(self.repository.as_ref(), zone_slug).await?;

        let removed = self.repository.delete_reading(*zone.id(), month).await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            zone_slug = zone_slug.as_str(),
            month = %String::from(month),
            removed,
            "zone reading deleted from the dashboard"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::AppError as GlobalAppError,
        features::zones::app::testing::{
            FakeZoneRepository, RepositoryCall, a_month, a_reading, a_slug, an_actor,
        },
    };

    #[tokio::test]
    async fn removes_only_the_reading_of_that_zone_and_month() {
        let repository = FakeZoneRepository::seeded().with_readings(vec![
            a_reading(2, "2026-03", 40),
            a_reading(2, "2026-02", 41),
            a_reading(1, "2026-03", 42),
        ]);
        let use_case = DeleteZoneReadingUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(&an_actor(7), &a_slug("kalar"), a_month("2026-03"))
            .await
            .expect("deleted");

        assert_eq!(
            repository
                .stored_readings()
                .iter()
                .map(|reading| reading.dryness().value())
                .collect::<Vec<_>>(),
            vec![41, 42]
        );
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindZoneBySlug {
                    slug: "kalar".to_string(),
                },
                RepositoryCall::DeleteReading {
                    zone_id: 2,
                    month: "2026-03".to_string(),
                },
            ]
        );
    }

    #[tokio::test]
    async fn deleting_what_is_already_gone_succeeds_again() {
        let repository =
            FakeZoneRepository::seeded().with_readings(vec![a_reading(2, "2026-03", 40)]);
        let use_case = DeleteZoneReadingUseCase::new(Arc::new(repository.clone()));

        for _ in 0..2 {
            use_case
                .execute(&an_actor(7), &a_slug("kalar"), a_month("2026-03"))
                .await
                .expect("deleted");
        }

        assert!(repository.stored_readings().is_empty());
    }

    #[tokio::test]
    async fn an_unknown_zone_is_not_found_and_nothing_is_written() {
        let repository = FakeZoneRepository::seeded();
        let use_case = DeleteZoneReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&an_actor(7), &a_slug("atlantis"), a_month("2026-03"))
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(!repository.wrote());
    }
}
