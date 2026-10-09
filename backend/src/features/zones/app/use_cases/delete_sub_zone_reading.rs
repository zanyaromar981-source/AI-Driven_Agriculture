use std::sync::Arc;

use super::locate::sub_zone_named;
use crate::{
    app::StaffContext,
    features::zones::{
        app::{AppError, ZoneRepository},
        domain::{Month, ZoneSlug},
    },
};

pub struct DeleteSubZoneReadingUseCase {
    repository: Arc<dyn ZoneRepository>,
}

impl DeleteSubZoneReadingUseCase {
    pub fn new(repository: Arc<dyn ZoneRepository>) -> Self {
        Self { repository }
    }

    /// Removes the sub-zone's reading for the month. Deleting what is
    /// already gone succeeds.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        zone_slug: &ZoneSlug,
        sub_zone_slug: &ZoneSlug,
        month: Month,
    ) -> Result<(), AppError> {
        let sub_zone = sub_zone_named(self.repository.as_ref(), zone_slug, sub_zone_slug).await?;

        let removed = self
            .repository
            .delete_sub_zone_reading(*sub_zone.id(), month)
            .await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            zone_slug = zone_slug.as_str(),
            sub_zone_slug = sub_zone_slug.as_str(),
            month = %String::from(month),
            removed,
            "sub-zone reading deleted from the dashboard"
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
            FakeZoneRepository, RepositoryCall, a_month, a_slug, a_sub_zone_reading, an_actor,
        },
    };

    #[tokio::test]
    async fn removes_only_the_reading_of_that_sub_zone_and_month() {
        let repository = FakeZoneRepository::seeded().with_sub_zone_readings(vec![
            a_sub_zone_reading(3, "2026-03", 20),
            a_sub_zone_reading(3, "2026-02", 21),
            a_sub_zone_reading(2, "2026-03", 22),
        ]);
        let use_case = DeleteSubZoneReadingUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(
                &an_actor(7),
                &a_slug("chamchamal"),
                &a_slug("sangaw"),
                a_month("2026-03"),
            )
            .await
            .expect("deleted");

        assert_eq!(
            repository
                .stored_sub_zone_readings()
                .iter()
                .map(|reading| reading.dryness().value())
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
        assert_eq!(
            repository.calls().last(),
            Some(&RepositoryCall::DeleteSubZoneReading {
                sub_zone_id: 3,
                month: "2026-03".to_string(),
            })
        );
    }

    #[tokio::test]
    async fn deleting_what_is_already_gone_succeeds_again() {
        let repository = FakeZoneRepository::seeded();
        let use_case = DeleteSubZoneReadingUseCase::new(Arc::new(repository.clone()));

        for _ in 0..2 {
            use_case
                .execute(
                    &an_actor(7),
                    &a_slug("chamchamal"),
                    &a_slug("sangaw"),
                    a_month("2026-03"),
                )
                .await
                .expect("deleted");
        }
    }

    #[tokio::test]
    async fn a_sub_zone_of_another_zone_is_not_found_and_nothing_is_written() {
        let repository = FakeZoneRepository::seeded();
        let use_case = DeleteSubZoneReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(
                &an_actor(7),
                &a_slug("kalar"),
                &a_slug("sangaw"),
                a_month("2026-03"),
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(!repository.wrote());
    }
}
