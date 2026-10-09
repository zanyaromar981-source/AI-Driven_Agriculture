use std::sync::Arc;

use crate::features::water::{
    app::{AppError, WaterPlanRepository},
    domain::Season,
};

pub struct ListWaterSeasonsUseCase {
    repository: Arc<dyn WaterPlanRepository>,
}

impl ListWaterSeasonsUseCase {
    pub fn new(repository: Arc<dyn WaterPlanRepository>) -> Self {
        Self { repository }
    }

    /// Returns the seasons that have at least one plan entry, newest first.
    pub async fn execute(&self) -> Result<Vec<Season>, AppError> {
        let seasons = self.repository.find_seasons().await?;

        tracing::debug!(returned = seasons.len(), "water plan seasons listed");

        Ok(seasons)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::water::app::testing::{FakeWaterPlanRepository, RepositoryCall, an_entry};

    #[tokio::test]
    async fn each_season_with_an_entry_is_listed_once_newest_first() {
        let repository = FakeWaterPlanRepository::holding(vec![
            an_entry("2025-26", "erbil", 99.0, None, None, false),
            an_entry("2026-27", "koya", 60.0, None, None, false),
            an_entry("2026-27", "makhmur", 90.0, None, None, false),
        ]);
        let use_case = ListWaterSeasonsUseCase::new(Arc::new(repository.clone()));

        let seasons = use_case.execute().await.expect("seasons");

        assert_eq!(
            seasons.iter().map(Season::as_str).collect::<Vec<_>>(),
            vec!["2026-27", "2025-26"]
        );
        assert_eq!(repository.calls(), vec![RepositoryCall::FindSeasons]);
    }

    #[tokio::test]
    async fn with_nothing_stored_the_list_is_empty_not_an_error() {
        let use_case = ListWaterSeasonsUseCase::new(Arc::new(FakeWaterPlanRepository::new()));

        assert!(use_case.execute().await.expect("seasons").is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListWaterSeasonsUseCase::new(Arc::new(FakeWaterPlanRepository::failing()));

        assert!(use_case.execute().await.is_err());
    }
}
