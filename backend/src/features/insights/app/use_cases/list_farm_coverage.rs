use std::sync::Arc;

use crate::features::insights::{
    app::{AppError, FarmDirectory, InsightRepository},
    domain::FarmCoverage,
};

pub struct ListFarmCoverageUseCase {
    repository: Arc<dyn InsightRepository>,
    directory: Arc<dyn FarmDirectory>,
}

impl ListFarmCoverageUseCase {
    pub fn new(repository: Arc<dyn InsightRepository>, directory: Arc<dyn FarmDirectory>) -> Self {
        Self {
            repository,
            directory,
        }
    }

    /// Returns every farm of every farmer with the day of each reading it
    /// already has, so a data job knows which readings to compute. No owner
    /// is part of the answer.
    pub async fn execute(&self) -> Result<Vec<FarmCoverage>, AppError> {
        let sites = self.directory.all_sites().await?;
        let stamps = self.repository.find_all_stamps().await?;

        let coverage = FarmCoverage::assemble(sites, stamps);

        tracing::debug!(farms = coverage.len(), "farm coverage listed");

        Ok(coverage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::insights::{
        app::testing::{Call, Fakes, a_day, an_insight},
        domain::Topic,
    };

    fn use_case(fakes: &Fakes) -> ListFarmCoverageUseCase {
        ListFarmCoverageUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn lists_every_farm_with_the_day_of_each_reading_it_has() {
        let fakes = Fakes::new()
            .with_site(1)
            .with_site(2)
            .with_stored(an_insight(2, Topic::Weather, 8))
            .with_stored(an_insight(2, Topic::Soil, 5));

        let coverage = use_case(&fakes).execute().await.expect("coverage");

        assert_eq!(coverage.len(), 2);
        assert_eq!(*coverage[0].site().farm_id(), 1);
        assert!(coverage[0].readings().is_empty());
        assert_eq!(
            coverage[1].readings(),
            &vec![(Topic::Soil, a_day(5)), (Topic::Weather, a_day(8))]
        );
    }

    #[tokio::test]
    async fn asks_the_farms_feature_for_the_farms_and_its_own_table_for_the_days() {
        let fakes = Fakes::new().with_site(1);

        use_case(&fakes).execute().await.expect("coverage");

        assert_eq!(fakes.calls(), vec![Call::AllSites, Call::FindAllStamps]);
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
