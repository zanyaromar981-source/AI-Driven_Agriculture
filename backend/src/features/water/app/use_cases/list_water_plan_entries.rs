use std::sync::Arc;

use crate::features::water::{
    app::{AppError, WaterPlanRepository},
    domain::{Season, WaterPlan, WaterPlanEntry},
};

pub struct ListWaterPlanEntriesUseCase {
    repository: Arc<dyn WaterPlanRepository>,
}

impl ListWaterPlanEntriesUseCase {
    pub fn new(repository: Arc<dyn WaterPlanRepository>) -> Self {
        Self { repository }
    }

    /// Returns the season's stored entries, highest need first. The order
    /// is the plan's own ranking, so the editing screen lists the zones as
    /// the public plan does. A season without entries is an empty list.
    pub async fn execute(&self, season: Season) -> Result<Vec<WaterPlanEntry>, AppError> {
        let entries = self.repository.find_by_season(&season).await?;

        tracing::debug!(
            season = season.as_str(),
            entries = entries.len(),
            "water plan entries listed"
        );

        Ok(WaterPlan::new(season, entries)
            .entries()
            .iter()
            .map(|ranked| ranked.entry().clone())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::water::app::testing::{
        FakeWaterPlanRepository, RepositoryCall, an_entry, season,
    };

    fn repository() -> FakeWaterPlanRepository {
        FakeWaterPlanRepository::holding(vec![
            an_entry("2025-26", "erbil", 99.0, None, None, false),
            an_entry("2026-27", "koya", 60.0, None, None, false),
            an_entry("2026-27", "makhmur", 90.0, Some("dukan"), Some(40.0), true),
            an_entry("2026-27", "chamchamal", 60.0, None, None, false),
        ])
    }

    #[tokio::test]
    async fn the_seasons_entries_come_highest_need_first_and_level_ones_by_slug() {
        let repository = repository();
        let use_case = ListWaterPlanEntriesUseCase::new(Arc::new(repository.clone()));

        let entries = use_case.execute(season("2026-27")).await.expect("entries");

        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.zone_slug().as_str())
                .collect::<Vec<_>>(),
            vec!["makhmur", "chamchamal", "koya"],
            "another season's entry must not be listed"
        );
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindBySeason {
                season: "2026-27".to_string()
            }]
        );
    }

    #[tokio::test]
    async fn a_season_without_entries_is_an_empty_list() {
        let use_case = ListWaterPlanEntriesUseCase::new(Arc::new(repository()));

        assert!(
            use_case
                .execute(season("2030-31"))
                .await
                .expect("entries")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case =
            ListWaterPlanEntriesUseCase::new(Arc::new(FakeWaterPlanRepository::failing()));

        assert!(use_case.execute(season("2026-27")).await.is_err());
    }
}
