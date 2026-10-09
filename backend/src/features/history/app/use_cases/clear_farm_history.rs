use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::history::{
        app::{AppError, HistoryRepository},
        domain::Metric,
    },
};

pub struct ClearFarmHistoryUseCase {
    repository: Arc<dyn HistoryRepository>,
}

impl ClearFarmHistoryUseCase {
    pub fn new(repository: Arc<dyn HistoryRepository>) -> Self {
        Self { repository }
    }

    /// Removes one metric's whole series from a farm, so the data job sees
    /// it as missing and fetches it again. A series that is already gone is
    /// a success, so a repeated delete gets the same answer as the first.
    /// The farm is not looked up, which also lets staff clear months left
    /// behind by a deleted farm.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        farm_id: i32,
        metric: Metric,
    ) -> Result<(), AppError> {
        self.repository.delete_series(farm_id, metric).await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            farm_id,
            metric = %String::from(metric),
            "farm history series cleared by staff"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::history::app::testing::{Call, FARM_ID, Fakes, a_series, actor};

    #[tokio::test]
    async fn clears_only_that_metric_of_that_farm_in_one_call() {
        let fakes = Fakes::new()
            .with_stored(FARM_ID, a_series(Metric::Greenness, &[("2026-04", 0.6)]))
            .with_stored(FARM_ID, a_series(Metric::RainMm, &[("2026-01", 120.0)]))
            .with_stored(99, a_series(Metric::Greenness, &[("2026-04", 0.5)]));
        let use_case = ClearFarmHistoryUseCase::new(Arc::new(fakes.clone()));

        use_case
            .execute(&actor(), FARM_ID, Metric::Greenness)
            .await
            .expect("cleared");

        assert!(fakes.stored(FARM_ID, Metric::Greenness).is_none());
        assert!(fakes.stored(FARM_ID, Metric::RainMm).is_some());
        assert!(fakes.stored(99, Metric::Greenness).is_some());
        assert_eq!(
            fakes.calls(),
            vec![Call::DeleteSeries {
                farm_id: FARM_ID,
                metric: Metric::Greenness,
            }]
        );
    }

    #[tokio::test]
    async fn clearing_a_series_that_is_already_gone_succeeds_again() {
        let use_case = ClearFarmHistoryUseCase::new(Arc::new(Fakes::new()));

        for _ in 0..2 {
            use_case
                .execute(&actor(), FARM_ID, Metric::Greenness)
                .await
                .expect("cleared");
        }
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ClearFarmHistoryUseCase::new(Arc::new(Fakes::new().failing()));

        assert!(
            use_case
                .execute(&actor(), FARM_ID, Metric::Greenness)
                .await
                .is_err()
        );
    }
}
