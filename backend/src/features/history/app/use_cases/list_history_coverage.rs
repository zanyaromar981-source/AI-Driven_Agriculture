use std::sync::Arc;

use crate::features::history::{
    app::{AppError, HistoryFarmDirectory, HistoryRepository},
    domain::FarmCoverage,
};

pub struct ListHistoryCoverageUseCase {
    repository: Arc<dyn HistoryRepository>,
    directory: Arc<dyn HistoryFarmDirectory>,
}

impl ListHistoryCoverageUseCase {
    pub fn new(
        repository: Arc<dyn HistoryRepository>,
        directory: Arc<dyn HistoryFarmDirectory>,
    ) -> Self {
        Self {
            repository,
            directory,
        }
    }

    /// Returns every farm of every farmer with how much of each metric is
    /// stored for it, so the data job knows what is missing. No owner is
    /// part of the answer.
    pub async fn execute(&self) -> Result<Vec<FarmCoverage>, AppError> {
        let sites = self.directory.all_sites().await?;
        let stored = self.repository.find_all_coverage().await?;

        let coverage = FarmCoverage::assemble(sites, stored);

        tracing::debug!(farms = coverage.len(), "history coverage listed");

        Ok(coverage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::history::{
        app::testing::{Call, Fakes, a_series},
        domain::Metric,
    };

    fn use_case(fakes: &Fakes) -> ListHistoryCoverageUseCase {
        ListHistoryCoverageUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn lists_every_farm_with_the_span_of_each_metric_it_has() {
        let fakes = Fakes::new().with_site(1).with_site(2).with_stored(
            2,
            a_series(
                Metric::RainMm,
                &[("2016-10", 31.2), ("2016-12", 80.0), ("2026-09", 0.0)],
            ),
        );

        let coverage = use_case(&fakes).execute().await.expect("coverage");

        assert_eq!(coverage.len(), 2);
        assert!(coverage[0].metrics().is_empty());

        let rain = coverage[1].metrics()[0];

        assert_eq!(rain.first_month().to_string(), "2016-10");
        assert_eq!(rain.last_month().to_string(), "2026-09");
        assert_eq!(
            *rain.months(),
            3,
            "the count shows the job a hole the two ends hide"
        );
    }

    #[tokio::test]
    async fn asks_the_farms_feature_for_the_farms_and_its_own_tables_for_the_spans() {
        let fakes = Fakes::new().with_site(1);

        use_case(&fakes).execute().await.expect("coverage");

        assert_eq!(fakes.calls(), vec![Call::AllSites, Call::FindAllCoverage]);
    }

    #[tokio::test]
    async fn no_farms_is_an_empty_list_not_an_error() {
        let coverage = use_case(&Fakes::new()).execute().await.expect("coverage");

        assert!(coverage.is_empty());
    }

    #[tokio::test]
    async fn a_failure_surfaces() {
        assert!(use_case(&Fakes::new().failing()).execute().await.is_err());
    }
}
