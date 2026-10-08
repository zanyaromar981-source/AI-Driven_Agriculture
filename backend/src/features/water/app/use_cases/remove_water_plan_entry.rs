use std::sync::Arc;

use crate::features::water::{
    app::{AppError, WaterPlanRepository},
    domain::{Season, ZoneSlug},
};

pub struct RemoveWaterPlanEntryUseCase {
    repository: Arc<dyn WaterPlanRepository>,
}

impl RemoveWaterPlanEntryUseCase {
    pub fn new(repository: Arc<dyn WaterPlanRepository>) -> Self {
        Self { repository }
    }

    /// Takes one zone out of a season's plan. Removing an entry that is not
    /// there succeeds: a job that runs again, or repeats a call whose answer
    /// was lost, asked for the entry to be gone, and it is.
    pub async fn execute(&self, season: Season, zone_slug: ZoneSlug) -> Result<(), AppError> {
        let removed = self.repository.delete(&season, &zone_slug).await?;

        tracing::info!(
            season = season.as_str(),
            zone = zone_slug.as_str(),
            removed,
            "water plan entry removed, or already gone"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::water::app::testing::{
        FakeWaterPlanRepository, RepositoryCall, an_entry, season, zone_slug,
    };

    #[tokio::test]
    async fn removes_the_entry_of_the_named_season_and_zone() {
        let repository = FakeWaterPlanRepository::holding(vec![an_entry(
            "2026-27", "makhmur", 90.0, None, None, false,
        )]);
        let use_case = RemoveWaterPlanEntryUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(season("2026-27"), zone_slug("makhmur"))
            .await;

        assert!(result.is_ok());
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
    async fn removing_an_entry_that_is_already_gone_succeeds() {
        let repository = FakeWaterPlanRepository::holding(vec![an_entry(
            "2026-27", "makhmur", 90.0, None, None, false,
        )]);
        let use_case = RemoveWaterPlanEntryUseCase::new(Arc::new(repository));

        let first = use_case
            .execute(season("2026-27"), zone_slug("makhmur"))
            .await;
        let repeat = use_case
            .execute(season("2026-27"), zone_slug("makhmur"))
            .await;

        assert!(first.is_ok());
        assert!(
            repeat.is_ok(),
            "a job that runs twice must not fail on its second pass"
        );
    }
}
