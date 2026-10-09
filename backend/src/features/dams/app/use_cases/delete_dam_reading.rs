use std::sync::Arc;

use chrono::NaiveDate;

use crate::{
    app::StaffContext,
    features::dams::{
        app::{AppError, DamRepository},
        domain::DamSlug,
    },
};

pub struct DeleteDamReadingUseCase {
    repository: Arc<dyn DamRepository>,
}

impl DeleteDamReadingUseCase {
    pub fn new(repository: Arc<dyn DamRepository>) -> Self {
        Self { repository }
    }

    /// Removes one dam's reading for one day. Removing a reading that is
    /// not there succeeds: a repeat of a call whose answer was lost asked
    /// for the reading to be gone, and it is. A dam we do not know is still
    /// not found.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        slug: DamSlug,
        day: NaiveDate,
    ) -> Result<(), AppError> {
        let dam = self
            .repository
            .find_by_slug(&slug)
            .await?
            .ok_or_else(|| AppError::DamNotFound(String::from(&slug)))?;

        let removed = self.repository.delete_reading(*dam.id(), day).await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            dam = slug.as_str(),
            day = %day,
            removed,
            "dam reading deleted from the dashboard, or already gone"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::dams::app::testing::{
        FakeDamRepository, RepositoryCall, a_reading, darbandikhan, day, dukan, slug, staff,
    };

    fn repository() -> FakeDamRepository {
        FakeDamRepository::holding(
            vec![dukan(), darbandikhan()],
            vec![
                a_reading(&dukan(), day(2026, 10, 1), 61.0),
                a_reading(&darbandikhan(), day(2026, 10, 1), 20.0),
            ],
        )
    }

    #[tokio::test]
    async fn removes_the_reading_of_the_named_dam_and_day_only() {
        let repository = repository();
        let use_case = DeleteDamReadingUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(&staff(), slug("dukan"), day(2026, 10, 1))
            .await
            .expect("deleted");

        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindBySlug {
                    slug: "dukan".to_string()
                },
                RepositoryCall::DeleteReading {
                    dam_id: 1,
                    day: day(2026, 10, 1),
                },
            ],
            "the dam must be part of the delete, never the day alone"
        );
    }

    #[tokio::test]
    async fn deleting_a_reading_that_is_already_gone_succeeds() {
        let use_case = DeleteDamReadingUseCase::new(Arc::new(repository()));

        let first = use_case
            .execute(&staff(), slug("dukan"), day(2026, 10, 1))
            .await;
        let repeat = use_case
            .execute(&staff(), slug("dukan"), day(2026, 10, 1))
            .await;

        assert!(first.is_ok());
        assert!(repeat.is_ok(), "a repeated delete must not fail");
    }

    #[tokio::test]
    async fn an_unknown_dam_is_not_found_and_nothing_is_deleted() {
        let repository = repository();
        let use_case = DeleteDamReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&staff(), slug("mosul"), day(2026, 10, 1))
            .await;

        assert!(matches!(result, Err(AppError::DamNotFound(slug)) if slug == "mosul"));
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindBySlug {
                slug: "mosul".to_string()
            }]
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = DeleteDamReadingUseCase::new(Arc::new(FakeDamRepository::failing()));

        assert!(
            use_case
                .execute(&staff(), slug("dukan"), day(2026, 10, 1))
                .await
                .is_err()
        );
    }
}
