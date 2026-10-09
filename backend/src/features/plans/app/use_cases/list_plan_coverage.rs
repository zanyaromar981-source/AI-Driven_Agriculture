use std::sync::Arc;

use crate::features::plans::{
    app::{AppError, PlanFarms, PlanRepository},
    domain::PlanCoverage,
};

pub struct ListPlanCoverageUseCase {
    repository: Arc<dyn PlanRepository>,
    farms: Arc<dyn PlanFarms>,
}

impl ListPlanCoverageUseCase {
    pub fn new(repository: Arc<dyn PlanRepository>, farms: Arc<dyn PlanFarms>) -> Self {
        Self { repository, farms }
    }

    /// Returns every farm of every farmer with the time its plan was issued,
    /// so the data job knows which farms need a new one. No owner is part of
    /// the answer.
    pub async fn execute(&self) -> Result<Vec<PlanCoverage>, AppError> {
        let sites = self.farms.all_sites().await?;
        let stamps = self.repository.find_all_stamps().await?;

        let coverage = PlanCoverage::assemble(sites, stamps);

        tracing::debug!(farms = coverage.len(), "plan coverage listed");

        Ok(coverage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::plans::app::testing::{Call, Fakes, a_plan, a_time};

    fn use_case(fakes: &Fakes) -> ListPlanCoverageUseCase {
        ListPlanCoverageUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn lists_every_farm_with_the_time_its_plan_was_issued() {
        let fakes = Fakes::new()
            .with_site(1)
            .with_site(2)
            .with_stored(a_plan(2, 8));

        let coverage = use_case(&fakes).execute().await.expect("coverage");

        assert_eq!(coverage.len(), 2);
        assert_eq!(*coverage[0].site().farm_id(), 1);
        assert!(coverage[0].issued().is_none());
        assert_eq!(*coverage[1].issued(), Some(a_time(8, 6)));
    }

    #[tokio::test]
    async fn asks_the_farms_feature_for_the_farms_and_its_own_table_for_the_times() {
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
