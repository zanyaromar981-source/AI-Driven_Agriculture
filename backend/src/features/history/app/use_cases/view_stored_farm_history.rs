use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::{
    app::AppError as GlobalAppError,
    features::history::{
        app::{AppError, HistoryFarmDirectory, HistoryRepository, use_cases::FarmHistoryQuery},
        domain::Series,
    },
};

pub struct ViewStoredFarmHistoryUseCase {
    repository: Arc<dyn HistoryRepository>,
    directory: Arc<dyn HistoryFarmDirectory>,
}

impl ViewStoredFarmHistoryUseCase {
    pub fn new(
        repository: Arc<dyn HistoryRepository>,
        directory: Arc<dyn HistoryFarmDirectory>,
    ) -> Self {
        Self {
            repository,
            directory,
        }
    }

    /// Returns the stored series of any farmer's farm for the dashboard, in
    /// the order the metrics are declared. A farm that does not exist is
    /// not found; one with nothing fetched yet gives an empty list.
    pub async fn execute(
        &self,
        farm_id: i32,
        query: FarmHistoryQuery,
        now: DateTime<Utc>,
    ) -> Result<Vec<Series>, AppError> {
        let window = query.window(now)?;

        if !self.directory.exists(farm_id).await? {
            return Err(GlobalAppError::NotFound.into());
        }

        let mut series = self
            .repository
            .find_series(farm_id, &query.metrics(), &window)
            .await?;
        series.sort_by_key(|series| *series.metric());

        tracing::debug!(
            farm_id,
            returned = series.len(),
            "stored farm history viewed"
        );

        Ok(series)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::history::{
        app::testing::{Call, FARM_ID, Fakes, a_series, now},
        domain::Metric,
    };

    fn use_case(fakes: &Fakes) -> ViewStoredFarmHistoryUseCase {
        ViewStoredFarmHistoryUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn returns_the_series_of_any_existing_farm_in_metric_order() {
        let fakes = Fakes::new()
            .with_site(99)
            .with_stored(99, a_series(Metric::Greenness, &[("2026-04", 0.6)]))
            .with_stored(99, a_series(Metric::RainMm, &[("2026-01", 120.0)]))
            .with_stored(FARM_ID, a_series(Metric::Et0Mm, &[("2026-01", 30.0)]));

        let series = use_case(&fakes)
            .execute(99, FarmHistoryQuery::default(), now())
            .await
            .expect("series");

        let metrics: Vec<Metric> = series.iter().map(|series| *series.metric()).collect();

        assert_eq!(metrics, vec![Metric::RainMm, Metric::Greenness]);
        assert!(
            matches!(
                fakes.calls().as_slice(),
                [
                    Call::Exists { farm_id: 99 },
                    Call::FindSeries { farm_id: 99, .. }
                ]
            ),
            "no owner is asked for: staff may read any farmer's farm"
        );
    }

    #[tokio::test]
    async fn a_farm_with_nothing_fetched_yet_is_an_empty_list_not_an_error() {
        let fakes = Fakes::new().with_site(FARM_ID);

        assert!(
            use_case(&fakes)
                .execute(FARM_ID, FarmHistoryQuery::default(), now())
                .await
                .expect("series")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn a_farm_that_does_not_exist_is_not_found_even_with_months_left_behind() {
        let fakes =
            Fakes::new().with_stored(FARM_ID, a_series(Metric::RainMm, &[("2026-01", 120.0)]));

        let result = use_case(&fakes)
            .execute(FARM_ID, FarmHistoryQuery::default(), now())
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert_eq!(fakes.calls(), vec![Call::Exists { farm_id: FARM_ID }]);
    }

    #[tokio::test]
    async fn a_failed_existence_check_surfaces_instead_of_reading() {
        let fakes = Fakes::new().with_site(FARM_ID).failing();

        assert!(
            use_case(&fakes)
                .execute(FARM_ID, FarmHistoryQuery::default(), now())
                .await
                .is_err()
        );
        assert_eq!(fakes.calls().len(), 1);
    }
}
