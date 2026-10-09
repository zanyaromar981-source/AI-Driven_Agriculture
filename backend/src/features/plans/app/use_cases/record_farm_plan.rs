use std::sync::Arc;

use chrono::{DateTime, NaiveDate, Utc};

use crate::{
    app::AppError as GlobalAppError,
    features::plans::{
        app::{AppError, PlanFarms, PlanRepository},
        domain::{DailyValues, FarmPlan, PlanAlert, PlanDecision, PlanSource},
    },
};

pub struct RecordFarmPlanInput {
    pub farm_id: i32,
    pub from: NaiveDate,
    pub issued: DateTime<Utc>,
    pub daily: DailyValues,
    pub alerts: Vec<PlanAlert>,
    pub decisions: Vec<PlanDecision>,
    pub source: PlanSource,
}

pub struct RecordFarmPlanUseCase {
    repository: Arc<dyn PlanRepository>,
    farms: Arc<dyn PlanFarms>,
}

impl RecordFarmPlanUseCase {
    pub fn new(repository: Arc<dyn PlanRepository>, farms: Arc<dyn PlanFarms>) -> Self {
        Self { repository, farms }
    }

    /// Stores the data job's plan as the farm's current one and returns the
    /// plan that is stored afterwards: the pushed one, or the one already
    /// there when that was issued later. A farm that does not exist is not
    /// found.
    pub async fn execute(
        &self,
        input: RecordFarmPlanInput,
        now: DateTime<Utc>,
    ) -> Result<FarmPlan, AppError> {
        let plan = FarmPlan::new(
            input.farm_id,
            input.from,
            input.issued,
            input.daily,
            input.alerts,
            input.decisions,
            input.source,
            now,
        )?;

        // Only for the answer: a farm deleted between this and the write
        // leaves a plan nobody is ever served, as every read starts from a
        // farm that exists.
        if !self.farms.exists(input.farm_id).await? {
            tracing::info!(farm_id = input.farm_id, "plan refused: no such farm");

            return Err(GlobalAppError::NotFound.into());
        }

        let stored = self.repository.upsert(&plan).await?;

        tracing::info!(
            farm_id = *stored.farm_id(),
            issued = %stored.issued(),
            kept_newer = stored.issued() != plan.issued(),
            alerts = stored.alerts().len(),
            decisions = stored.decisions().len(),
            "farm plan recorded"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::plans::{
        app::testing::{
            Call, FARM_ID, Fakes, a_day, a_decision, a_plan, a_source, a_time, an_alert, ten_days,
        },
        domain::PlanError,
    };

    fn input(alerts: Vec<PlanAlert>) -> RecordFarmPlanInput {
        RecordFarmPlanInput {
            farm_id: FARM_ID,
            from: a_day(8),
            issued: a_time(8, 12),
            daily: ten_days(),
            alerts,
            decisions: vec![a_decision()],
            source: a_source(),
        }
    }

    fn use_case(fakes: &Fakes) -> RecordFarmPlanUseCase {
        RecordFarmPlanUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn stores_the_plan_for_a_farm_that_exists() {
        let fakes = Fakes::new().with_site(FARM_ID);

        let stored = use_case(&fakes)
            .execute(input(vec![an_alert(11)]), a_time(8, 12))
            .await
            .expect("plan");

        assert!(stored.id().is_some(), "the answer is the stored plan");
        assert_eq!(*stored.issued(), a_time(8, 12));
        assert_eq!(
            fakes.calls(),
            vec![
                Call::Exists { farm_id: FARM_ID },
                Call::Upsert { farm_id: FARM_ID },
            ]
        );
    }

    #[tokio::test]
    async fn a_newer_plan_replaces_the_one_the_farm_had() {
        let fakes = Fakes::new()
            .with_site(FARM_ID)
            .with_stored(a_plan(FARM_ID, 8));

        use_case(&fakes)
            .execute(input(vec![]), a_time(8, 12))
            .await
            .expect("plan");

        let stored = fakes.stored(FARM_ID).expect("stored");

        assert_eq!(*stored.issued(), a_time(8, 12));
        assert!(stored.alerts().is_empty(), "every part is replaced");
    }

    #[tokio::test]
    async fn an_older_plan_does_not_replace_a_newer_one_and_the_newer_is_the_answer() {
        let fakes = Fakes::new()
            .with_site(FARM_ID)
            .with_stored(a_plan(FARM_ID, 9));

        let answer = use_case(&fakes)
            .execute(input(vec![]), a_time(9, 12))
            .await
            .expect("plan");

        assert_eq!(*answer.issued(), a_time(9, 6));
        assert_eq!(
            *fakes.stored(FARM_ID).expect("stored").from(),
            a_day(9),
            "a slower copy of the job must not put an older plan back"
        );
    }

    #[tokio::test]
    async fn a_plan_for_a_farm_that_does_not_exist_is_not_found_and_not_written() {
        let fakes = Fakes::new();

        let result = use_case(&fakes).execute(input(vec![]), a_time(8, 12)).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert_eq!(fakes.calls(), vec![Call::Exists { farm_id: FARM_ID }]);
    }

    #[tokio::test]
    async fn a_plan_that_breaks_a_rule_touches_nothing() {
        let fakes = Fakes::new().with_site(FARM_ID);

        let result = use_case(&fakes)
            .execute(input(vec![an_alert(20)]), a_time(8, 12))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Plan(PlanError::AlertOutsidePlan(_)))
        ));
        assert!(fakes.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let fakes = Fakes::new().with_site(FARM_ID).failing();

        assert!(
            use_case(&fakes)
                .execute(input(vec![]), a_time(8, 12))
                .await
                .is_err()
        );
    }
}
