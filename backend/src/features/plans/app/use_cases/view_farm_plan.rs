use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::plans::{
        app::{AppError, PlanFarms, PlanRepository},
        domain::{FarmPlan, PlanError},
    },
};

pub struct ViewFarmPlanUseCase {
    repository: Arc<dyn PlanRepository>,
    farms: Arc<dyn PlanFarms>,
}

impl ViewFarmPlanUseCase {
    pub fn new(repository: Arc<dyn PlanRepository>, farms: Arc<dyn PlanFarms>) -> Self {
        Self { repository, farms }
    }

    /// Returns the plan of one of the farmer's own farms as it may be seen
    /// at `now`: days that are over are left out. A farm of another phone is
    /// answered like one that does not exist. A farm with no plan yet, or
    /// with one too old to show, has no plan to answer with.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        farm_id: i32,
        now: DateTime<Utc>,
    ) -> Result<FarmPlan, AppError> {
        if !self
            .farms
            .is_owned_by(farm_id, auth_context.user().phone())
            .await?
        {
            tracing::info!(farm_id, "plan refused: no such farm for this owner");

            return Err(GlobalAppError::NotFound.into());
        }

        let Some(stored) = self.repository.find_by_farm(farm_id).await? else {
            tracing::debug!(farm_id, "farm has no plan yet");

            return Err(PlanError::NotReady.into());
        };

        let seen = stored.seen_at(now).inspect_err(|_| {
            // A plan is replaced every six hours, so a stale one means the
            // job has not run for days.
            tracing::warn!(farm_id, from = %stored.from(), "stored plan is stale, not served");
        })?;

        tracing::debug!(farm_id, days = seen.daily().days(), "farm plan viewed");

        Ok(seen)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::plans::app::testing::{
        Call, FARM_ID, Fakes, OWNER, a_day, a_plan, a_time, auth_context,
    };

    fn use_case(fakes: &Fakes) -> ViewFarmPlanUseCase {
        ViewFarmPlanUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn returns_the_plan_of_a_farm_the_user_owns() {
        let fakes = Fakes::new()
            .with_owned_farm()
            .with_stored(a_plan(FARM_ID, 8));

        let plan = use_case(&fakes)
            .execute(&auth_context(), FARM_ID, a_time(8, 9))
            .await
            .expect("plan");

        assert_eq!(*plan.from(), a_day(8));
        assert_eq!(plan.daily().days(), 10);
    }

    #[tokio::test]
    async fn ownership_is_checked_for_the_signed_in_phone_before_anything_is_read() {
        let fakes = Fakes::new()
            .with_owned_farm()
            .with_stored(a_plan(FARM_ID, 8));

        use_case(&fakes)
            .execute(&auth_context(), FARM_ID, a_time(8, 9))
            .await
            .expect("plan");

        assert_eq!(
            fakes.calls(),
            vec![
                Call::IsOwnedBy {
                    farm_id: FARM_ID,
                    phone: OWNER.to_string(),
                },
                Call::FindByFarm { farm_id: FARM_ID },
            ]
        );
    }

    #[tokio::test]
    async fn is_not_found_when_the_user_does_not_own_the_farm_and_reads_nothing() {
        let fakes = Fakes::new().with_owned_farm().with_stored(a_plan(99, 8));

        let result = use_case(&fakes)
            .execute(&auth_context(), 99, a_time(8, 9))
            .await;

        assert!(
            matches!(
                result,
                Err(AppError::GlobalAppError(GlobalAppError::NotFound))
            ),
            "exactly the answer for a farm that does not exist, not 'no plan yet'"
        );
        assert!(
            !fakes.calls().contains(&Call::FindByFarm { farm_id: 99 }),
            "the plan of another farmer's farm must not even be loaded"
        );
    }

    #[tokio::test]
    async fn a_farm_with_no_plan_yet_is_not_ready() {
        let fakes = Fakes::new().with_owned_farm();

        let result = use_case(&fakes)
            .execute(&auth_context(), FARM_ID, a_time(8, 9))
            .await;

        assert!(matches!(result, Err(AppError::Plan(PlanError::NotReady))));
    }

    #[tokio::test]
    async fn a_day_later_the_plan_starts_today_not_yesterday() {
        let fakes = Fakes::new()
            .with_owned_farm()
            .with_stored(a_plan(FARM_ID, 8));

        let plan = use_case(&fakes)
            .execute(&auth_context(), FARM_ID, a_time(9, 9))
            .await
            .expect("plan");

        assert_eq!(*plan.from(), a_day(9));
        assert_eq!(plan.daily().days(), 9);
    }

    #[tokio::test]
    async fn a_plan_more_than_two_days_old_is_not_served() {
        let fakes = Fakes::new()
            .with_owned_farm()
            .with_stored(a_plan(FARM_ID, 8));

        let result = use_case(&fakes)
            .execute(&auth_context(), FARM_ID, a_time(11, 9))
            .await;

        assert!(matches!(result, Err(AppError::Plan(PlanError::Stale))));
    }

    #[tokio::test]
    async fn a_failed_ownership_check_surfaces_instead_of_reading() {
        let fakes = Fakes::new().with_owned_farm().failing();

        let result = use_case(&fakes)
            .execute(&auth_context(), FARM_ID, a_time(8, 9))
            .await;

        assert!(result.is_err());
        assert!(
            !fakes
                .calls()
                .contains(&Call::FindByFarm { farm_id: FARM_ID })
        );
    }
}
