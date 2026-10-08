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
    /// there is reported, so a job can tell a typo from a removal.
    pub async fn execute(&self, season: Season, zone_slug: ZoneSlug) -> Result<(), AppError> {
        let removed = self.repository.delete(&season, &zone_slug).await?;

        if !removed {
            return Err(AppError::EntryNotFound {
                season: String::from(&season),
                zone_slug: String::from(&zone_slug),
            });
        }

        tracing::info!(
            season = season.as_str(),
            zone = zone_slug.as_str(),
            "water plan entry removed"
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
    async fn removing_an_entry_that_is_not_there_is_not_found() {
        let repository = FakeWaterPlanRepository::holding(vec![an_entry(
            "2026-27", "makhmur", 90.0, None, None, false,
        )]);
        let use_case = RemoveWaterPlanEntryUseCase::new(Arc::new(repository));

        let other_season = use_case
            .execute(season("2025-26"), zone_slug("makhmur"))
            .await;
        let other_zone = use_case.execute(season("2026-27"), zone_slug("koya")).await;

        assert!(matches!(
            other_season,
            Err(AppError::EntryNotFound { season, zone_slug })
                if season == "2025-26" && zone_slug == "makhmur"
        ));
        assert!(matches!(other_zone, Err(AppError::EntryNotFound { .. })));
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case =
            RemoveWaterPlanEntryUseCase::new(Arc::new(FakeWaterPlanRepository::failing()));

        assert!(
            use_case
                .execute(season("2026-27"), zone_slug("makhmur"))
                .await
                .is_err()
        );
    }
}
