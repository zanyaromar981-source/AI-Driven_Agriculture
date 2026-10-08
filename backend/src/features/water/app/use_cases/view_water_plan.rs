use std::sync::Arc;

use crate::features::water::{
    app::{AppError, WaterPlanRepository},
    domain::{Season, WaterPlan},
};

pub struct ViewWaterPlanInput {
    /// None = the latest season that has a plan.
    pub season: Option<Season>,
}

pub struct ViewWaterPlanUseCase {
    repository: Arc<dyn WaterPlanRepository>,
}

impl ViewWaterPlanUseCase {
    pub fn new(repository: Arc<dyn WaterPlanRepository>) -> Self {
        Self { repository }
    }

    /// Returns the season's plan, ranked and totalled. A named season
    /// without entries is an empty plan. With no season named and nothing
    /// stored there is no plan at all.
    pub async fn execute(&self, input: ViewWaterPlanInput) -> Result<WaterPlan, AppError> {
        let season = match input.season {
            Some(season) => season,
            None => self
                .repository
                .find_latest_season()
                .await?
                .ok_or(AppError::NoWaterPlan)?,
        };

        let entries = self.repository.find_by_season(&season).await?;

        tracing::debug!(
            season = season.as_str(),
            entries = entries.len(),
            "water plan viewed"
        );

        Ok(WaterPlan::new(season, entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::water::app::testing::{
        FakeWaterPlanRepository, RepositoryCall, an_entry, season,
    };

    fn stored() -> FakeWaterPlanRepository {
        FakeWaterPlanRepository::holding(vec![
            an_entry("2025-26", "erbil", 99.0, Some("dukan"), Some(70.0), true),
            an_entry("2026-27", "koya", 60.0, Some("dukan"), Some(25.5), false),
            an_entry("2026-27", "makhmur", 90.0, Some("dukan"), Some(40.0), true),
            an_entry(
                "2026-27",
                "kalar",
                80.0,
                Some("darbandikhan"),
                Some(30.0),
                true,
            ),
        ])
    }

    #[tokio::test]
    async fn without_a_season_the_latest_plan_is_shown_ranked_and_totalled() {
        let repository = stored();
        let use_case = ViewWaterPlanUseCase::new(Arc::new(repository.clone()));

        let plan = use_case
            .execute(ViewWaterPlanInput { season: None })
            .await
            .expect("plan");

        assert_eq!(plan.season().as_str(), "2026-27");
        assert_eq!(
            plan.entries()
                .iter()
                .map(|ranked| (*ranked.rank(), ranked.entry().zone_slug().as_str()))
                .collect::<Vec<_>>(),
            vec![(1, "makhmur"), (2, "kalar"), (3, "koya")]
        );
        assert_eq!(*plan.totals().planned_million_m3(), 95.5);
        assert_eq!(*plan.totals().urgent_zones(), 2);
        assert_eq!(plan.totals().by_dam().len(), 2);
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindLatestSeason,
                RepositoryCall::FindBySeason {
                    season: "2026-27".to_string()
                },
            ]
        );
    }

    #[tokio::test]
    async fn a_named_season_is_shown_without_looking_for_the_latest() {
        let repository = stored();
        let use_case = ViewWaterPlanUseCase::new(Arc::new(repository.clone()));

        let plan = use_case
            .execute(ViewWaterPlanInput {
                season: Some(season("2025-26")),
            })
            .await
            .expect("plan");

        assert_eq!(plan.season().as_str(), "2025-26");
        assert_eq!(plan.entries().len(), 1);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindBySeason {
                season: "2025-26".to_string()
            }]
        );
    }

    #[tokio::test]
    async fn a_named_season_without_entries_is_an_empty_plan() {
        let use_case = ViewWaterPlanUseCase::new(Arc::new(stored()));

        let plan = use_case
            .execute(ViewWaterPlanInput {
                season: Some(season("2030-31")),
            })
            .await
            .expect("plan");

        assert!(plan.entries().is_empty());
        assert_eq!(*plan.totals().planned_million_m3(), 0.0);
    }

    #[tokio::test]
    async fn with_nothing_stored_there_is_no_plan_to_default_to() {
        let repository = FakeWaterPlanRepository::new();
        let use_case = ViewWaterPlanUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(ViewWaterPlanInput { season: None }).await;

        assert!(matches!(result, Err(AppError::NoWaterPlan)));
        assert_eq!(repository.calls(), vec![RepositoryCall::FindLatestSeason]);
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ViewWaterPlanUseCase::new(Arc::new(FakeWaterPlanRepository::failing()));

        assert!(
            use_case
                .execute(ViewWaterPlanInput { season: None })
                .await
                .is_err()
        );
    }
}
