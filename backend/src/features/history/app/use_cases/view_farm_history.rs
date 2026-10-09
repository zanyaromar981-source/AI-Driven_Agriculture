use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::history::{
        app::{AppError, HistoryFarmOwnership, HistoryRepository},
        domain::{HistoryWindow, Metric, Month, Series},
    },
};

/// What a reader asks of a farm's history. Everything left out means "all
/// of it": every metric, the last 120 full months.
#[derive(Clone, Debug, Default)]
pub struct FarmHistoryQuery {
    pub metrics: Option<Vec<Metric>>,
    pub from: Option<Month>,
    pub to: Option<Month>,
}

impl FarmHistoryQuery {
    pub(super) fn metrics(&self) -> Vec<Metric> {
        self.metrics.clone().unwrap_or_else(|| Metric::ALL.to_vec())
    }

    pub(super) fn window(&self, now: DateTime<Utc>) -> Result<HistoryWindow, AppError> {
        Ok(HistoryWindow::resolve(self.from, self.to, now)?)
    }
}

pub struct ViewFarmHistoryUseCase {
    repository: Arc<dyn HistoryRepository>,
    ownership: Arc<dyn HistoryFarmOwnership>,
}

impl ViewFarmHistoryUseCase {
    pub fn new(
        repository: Arc<dyn HistoryRepository>,
        ownership: Arc<dyn HistoryFarmOwnership>,
    ) -> Self {
        Self {
            repository,
            ownership,
        }
    }

    /// Returns the farm's stored series in the order the metrics are
    /// declared. A farm nothing has been fetched for yet gives an empty
    /// list. A farm of another phone is answered like one that does not
    /// exist.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        farm_id: i32,
        query: FarmHistoryQuery,
        now: DateTime<Utc>,
    ) -> Result<Vec<Series>, AppError> {
        let window = query.window(now)?;

        if !self
            .ownership
            .is_owned_by(farm_id, auth_context.user().phone())
            .await?
        {
            tracing::info!(farm_id, "history refused: no such farm for this owner");

            return Err(GlobalAppError::NotFound.into());
        }

        let mut series = self
            .repository
            .find_series(farm_id, &query.metrics(), &window)
            .await?;
        series.sort_by_key(|series| *series.metric());

        tracing::debug!(farm_id, returned = series.len(), "farm history viewed");

        Ok(series)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::history::{
        app::testing::{Call, FARM_ID, Fakes, OWNER, a_month, a_series, auth_context, now},
        domain::HistoryError,
    };

    fn use_case(fakes: &Fakes) -> ViewFarmHistoryUseCase {
        ViewFarmHistoryUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    fn everything() -> FarmHistoryQuery {
        FarmHistoryQuery::default()
    }

    #[tokio::test]
    async fn returns_the_series_of_a_farm_the_user_owns_in_metric_order() {
        let fakes = Fakes::new()
            .with_owned_farm()
            .with_stored(FARM_ID, a_series(Metric::RainMm, &[("2026-01", 120.0)]))
            .with_stored(FARM_ID, a_series(Metric::Greenness, &[("2026-04", 0.6)]))
            .with_stored(FARM_ID, a_series(Metric::TempMaxC, &[("2026-07", 41.0)]));

        let series = use_case(&fakes)
            .execute(&auth_context(), FARM_ID, everything(), now())
            .await
            .expect("series");

        let metrics: Vec<Metric> = series.iter().map(|series| *series.metric()).collect();

        assert_eq!(
            metrics,
            vec![Metric::RainMm, Metric::TempMaxC, Metric::Greenness]
        );
    }

    #[tokio::test]
    async fn ownership_is_checked_for_the_signed_in_phone_before_anything_is_read() {
        let fakes = Fakes::new().with_owned_farm();

        use_case(&fakes)
            .execute(&auth_context(), FARM_ID, everything(), now())
            .await
            .expect("series");

        assert_eq!(
            fakes.calls(),
            vec![
                Call::IsOwnedBy {
                    farm_id: FARM_ID,
                    phone: OWNER.to_string(),
                },
                Call::FindSeries {
                    farm_id: FARM_ID,
                    metrics: Metric::ALL.to_vec(),
                    from: "2016-10".to_string(),
                    to: "2026-09".to_string(),
                },
            ],
            "with nothing asked: every metric, the last 120 full months"
        );
    }

    #[tokio::test]
    async fn only_the_metrics_and_months_asked_for_are_read() {
        let fakes = Fakes::new()
            .with_owned_farm()
            .with_stored(
                FARM_ID,
                a_series(Metric::RainMm, &[("2019-12", 90.0), ("2020-01", 120.0)]),
            )
            .with_stored(FARM_ID, a_series(Metric::Greenness, &[("2020-01", 0.3)]));

        let series = use_case(&fakes)
            .execute(
                &auth_context(),
                FARM_ID,
                FarmHistoryQuery {
                    metrics: Some(vec![Metric::RainMm]),
                    from: Some(a_month("2020-01")),
                    to: Some(a_month("2020-12")),
                },
                now(),
            )
            .await
            .expect("series");

        assert_eq!(series.len(), 1);
        assert_eq!(*series[0].metric(), Metric::RainMm);
        assert_eq!(series[0].points().len(), 1, "December 2019 is outside");
    }

    #[tokio::test]
    async fn a_farm_with_nothing_fetched_yet_is_an_empty_list_not_an_error() {
        let fakes = Fakes::new().with_owned_farm();

        let series = use_case(&fakes)
            .execute(&auth_context(), FARM_ID, everything(), now())
            .await
            .expect("series");

        assert!(series.is_empty());
    }

    #[tokio::test]
    async fn is_not_found_when_the_user_does_not_own_the_farm_and_reads_nothing() {
        let fakes = Fakes::new()
            .with_owned_farm()
            .with_site(99)
            .with_stored(99, a_series(Metric::RainMm, &[("2026-01", 120.0)]));

        let result = use_case(&fakes)
            .execute(&auth_context(), 99, everything(), now())
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert_eq!(
            fakes.calls().len(),
            1,
            "the history of another farmer's farm must not even be loaded"
        );
    }

    #[tokio::test]
    async fn a_window_that_is_too_long_is_refused_before_anything_is_asked() {
        let fakes = Fakes::new().with_owned_farm();

        let result = use_case(&fakes)
            .execute(
                &auth_context(),
                FARM_ID,
                FarmHistoryQuery {
                    metrics: None,
                    from: Some(a_month("2000-01")),
                    to: None,
                },
                now(),
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::History(HistoryError::BadWindow { .. }))
        ));
        assert!(fakes.calls().is_empty());
    }

    #[tokio::test]
    async fn a_failed_ownership_check_surfaces_instead_of_reading() {
        let fakes = Fakes::new().with_owned_farm().failing();

        let result = use_case(&fakes)
            .execute(&auth_context(), FARM_ID, everything(), now())
            .await;

        assert!(result.is_err());
        assert_eq!(fakes.calls().len(), 1);
    }
}
