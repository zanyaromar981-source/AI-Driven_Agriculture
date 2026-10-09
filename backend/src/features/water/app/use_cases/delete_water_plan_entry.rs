use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::water::{
        app::{AppError, WaterPlanRepository},
        domain::{Season, ZoneSlug},
    },
};

pub struct DeleteWaterPlanEntryUseCase {
    repository: Arc<dyn WaterPlanRepository>,
}

impl DeleteWaterPlanEntryUseCase {
    pub fn new(repository: Arc<dyn WaterPlanRepository>) -> Self {
        Self { repository }
    }

    /// Takes one zone out of a season's plan on a staff member's word.
    /// Removing an entry that is not there succeeds: a repeat of a call
    /// whose answer was lost asked for the entry to be gone, and it is.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        season: Season,
        zone_slug: ZoneSlug,
    ) -> Result<(), AppError> {
        let removed = self.repository.delete(&season, &zone_slug).await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            season = season.as_str(),
            zone = zone_slug.as_str(),
            removed,
            "water plan entry deleted from the dashboard, or already gone"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::water::app::testing::{
        FakeWaterPlanRepository, RepositoryCall, an_entry, season, staff, zone_slug,
    };

    fn repository() -> FakeWaterPlanRepository {
        FakeWaterPlanRepository::holding(vec![an_entry(
            "2026-27", "makhmur", 90.0, None, None, false,
        )])
    }

    #[tokio::test]
    async fn removes_the_entry_of_the_named_season_and_zone() {
        let repository = repository();
        let use_case = DeleteWaterPlanEntryUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(&staff(), season("2026-27"), zone_slug("makhmur"))
            .await
            .expect("deleted");

        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::Delete {
                season: "2026-27".to_string(),
                zone_slug: "makhmur".to_string(),
            }],
            "the season must be part of the delete, never the zone alone"
        );
    }

    #[tokio::test]
    async fn deleting_an_entry_that_is_already_gone_succeeds() {
        let use_case = DeleteWaterPlanEntryUseCase::new(Arc::new(repository()));

        let actor = staff();
        let delete = || use_case.execute(&actor, season("2026-27"), zone_slug("makhmur"));

        assert!(delete().await.is_ok());
        assert!(delete().await.is_ok(), "a repeated delete must not fail");
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case =
            DeleteWaterPlanEntryUseCase::new(Arc::new(FakeWaterPlanRepository::failing()));

        assert!(
            use_case
                .execute(&staff(), season("2026-27"), zone_slug("makhmur"))
                .await
                .is_err()
        );
    }
}
