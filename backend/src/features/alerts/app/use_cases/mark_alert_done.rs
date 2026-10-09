use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::alerts::app::{AlertFarms, AlertRepository, AppError},
};

pub struct MarkAlertDoneUseCase {
    repository: Arc<dyn AlertRepository>,
    farms: Arc<dyn AlertFarms>,
}

impl MarkAlertDoneUseCase {
    pub fn new(repository: Arc<dyn AlertRepository>, farms: Arc<dyn AlertFarms>) -> Self {
        Self { repository, farms }
    }

    /// Ticks an alert of one of the farmer's own farms. Ticking it again
    /// succeeds and keeps the first time. An alert of another farmer's farm
    /// is answered like one that does not exist.
    pub async fn execute(&self, auth_context: &AuthContext, alert_id: i32) -> Result<(), AppError> {
        // This read only finds the farm to check the owner of. The tick
        // itself is one statement and safe to repeat.
        let Some(alert) = self.repository.find_by_id(alert_id).await? else {
            return Err(GlobalAppError::NotFound.into());
        };

        if !self
            .farms
            .is_owned_by(*alert.farm_id(), auth_context.user().phone())
            .await?
        {
            tracing::info!(alert_id, "tick refused: no such alert for this owner");

            return Err(GlobalAppError::NotFound.into());
        }

        if !self.repository.mark_done(alert_id, Utc::now()).await? {
            return Err(GlobalAppError::NotFound.into());
        }

        tracing::info!(alert_id, farm_id = *alert.farm_id(), "alert ticked as done");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alerts::app::testing::{
        Call, FARM_ID, Fakes, OTHER_FARM_ID, an_alert, auth_context,
    };

    fn use_case(fakes: &Fakes) -> MarkAlertDoneUseCase {
        MarkAlertDoneUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn ticks_an_alert_of_the_farmers_own_farm() {
        let fakes = Fakes::new().with_stored(an_alert(1, FARM_ID));

        use_case(&fakes)
            .execute(&auth_context(), 1)
            .await
            .expect("ticked");

        let stored = fakes.stored(1).expect("alert");

        assert!(*stored.done());
        assert!(stored.done_at().is_some());
    }

    #[tokio::test]
    async fn ticking_twice_succeeds_and_keeps_the_first_time() {
        let fakes = Fakes::new().with_stored(an_alert(1, FARM_ID));
        let use_case = use_case(&fakes);

        use_case.execute(&auth_context(), 1).await.expect("first");
        let first = *fakes.stored(1).expect("alert").done_at();

        use_case.execute(&auth_context(), 1).await.expect("second");

        assert_eq!(*fakes.stored(1).expect("alert").done_at(), first);
    }

    #[tokio::test]
    async fn another_farmers_alert_is_not_found_and_stays_unticked() {
        let fakes = Fakes::new().with_stored(an_alert(2, OTHER_FARM_ID));

        let error = use_case(&fakes)
            .execute(&auth_context(), 2)
            .await
            .expect_err("refused");

        assert!(matches!(
            error,
            AppError::GlobalAppError(GlobalAppError::NotFound)
        ));
        assert!(!*fakes.stored(2).expect("alert").done());
        assert!(
            !fakes
                .calls()
                .iter()
                .any(|call| matches!(call, Call::MarkDone { .. })),
            "the write must never be reached"
        );
    }

    #[tokio::test]
    async fn a_missing_alert_is_not_found() {
        let error = use_case(&Fakes::new())
            .execute(&auth_context(), 99)
            .await
            .expect_err("refused");

        assert!(matches!(
            error,
            AppError::GlobalAppError(GlobalAppError::NotFound)
        ));
    }
}
