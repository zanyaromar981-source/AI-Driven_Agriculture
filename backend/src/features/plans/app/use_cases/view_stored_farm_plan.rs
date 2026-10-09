use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::plans::{
        app::{AppError, PlanFarms, PlanRepository},
        domain::FarmPlan,
    },
};

pub struct ViewStoredFarmPlanUseCase {
    repository: Arc<dyn PlanRepository>,
    farms: Arc<dyn PlanFarms>,
}

impl ViewStoredFarmPlanUseCase {
    pub fn new(repository: Arc<dyn PlanRepository>, farms: Arc<dyn PlanFarms>) -> Self {
        Self { repository, farms }
    }

    /// Returns the plan of any farmer's farm for the dashboard, exactly as
    /// it is stored: past days are kept, so staff see what the job pushed. A
    /// farm that does not exist is not found; one with no plan yet gives
    /// `None`.
    pub async fn execute(&self, farm_id: i32) -> Result<Option<FarmPlan>, AppError> {
        if !self.farms.exists(farm_id).await? {
            return Err(GlobalAppError::NotFound.into());
        }

        let plan = self.repository.find_by_farm(farm_id).await?;

        tracing::debug!(farm_id, found = plan.is_some(), "stored farm plan viewed");

        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::plans::app::testing::{Call, FARM_ID, Fakes, a_day, a_plan};

    fn use_case(fakes: &Fakes) -> ViewStoredFarmPlanUseCase {
        ViewStoredFarmPlanUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn returns_the_stored_plan_of_any_existing_farm_untrimmed() {
        let fakes = Fakes::new().with_site(99).with_stored(a_plan(99, 1));

        let plan = use_case(&fakes)
            .execute(99)
            .await
            .expect("answer")
            .expect("plan");

        assert_eq!(*plan.from(), a_day(1), "however old, it is shown as stored");
        assert_eq!(plan.daily().days(), 10);
        assert_eq!(
            fakes.calls(),
            vec![
                Call::Exists { farm_id: 99 },
                Call::FindByFarm { farm_id: 99 }
            ],
            "no owner is asked for: staff may read any farmer's farm"
        );
    }

    #[tokio::test]
    async fn a_farm_with_no_plan_yet_is_none_not_an_error() {
        let fakes = Fakes::new().with_site(FARM_ID);

        assert!(
            use_case(&fakes)
                .execute(FARM_ID)
                .await
                .expect("answer")
                .is_none()
        );
    }

    #[tokio::test]
    async fn a_farm_that_does_not_exist_is_not_found_even_with_a_plan_left_behind() {
        let fakes = Fakes::new().with_stored(a_plan(FARM_ID, 8));

        let result = use_case(&fakes).execute(FARM_ID).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert_eq!(fakes.calls(), vec![Call::Exists { farm_id: FARM_ID }]);
    }

    #[tokio::test]
    async fn a_failed_existence_check_surfaces_instead_of_reading() {
        let fakes = Fakes::new().with_site(FARM_ID).failing();

        assert!(use_case(&fakes).execute(FARM_ID).await.is_err());
        assert!(
            !fakes
                .calls()
                .contains(&Call::FindByFarm { farm_id: FARM_ID })
        );
    }
}
