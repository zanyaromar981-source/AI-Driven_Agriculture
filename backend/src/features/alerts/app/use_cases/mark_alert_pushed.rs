use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::AppError as GlobalAppError,
    features::alerts::app::{AlertRepository, AppError},
};

pub struct MarkAlertPushedUseCase {
    repository: Arc<dyn AlertRepository>,
}

impl MarkAlertPushedUseCase {
    pub fn new(repository: Arc<dyn AlertRepository>) -> Self {
        Self { repository }
    }

    /// Records that the push sender sent an alert. Saying so again succeeds
    /// and keeps the first time, so the farm's one push a day is counted
    /// from when it really went out.
    pub async fn execute(&self, alert_id: i32) -> Result<(), AppError> {
        if !self.repository.mark_pushed(alert_id, Utc::now()).await? {
            return Err(GlobalAppError::NotFound.into());
        }

        tracing::info!(alert_id, "alert marked as pushed");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alerts::app::testing::{FARM_ID, Fakes, an_alert};

    #[tokio::test]
    async fn marks_the_alert_and_a_repeat_keeps_the_first_time() {
        let fakes = Fakes::new().with_stored(an_alert(1, FARM_ID));
        let use_case = MarkAlertPushedUseCase::new(Arc::new(fakes.clone()));

        use_case.execute(1).await.expect("first");
        let first = *fakes.stored(1).expect("alert").pushed_at();

        use_case.execute(1).await.expect("second");

        let stored = fakes.stored(1).expect("alert");

        assert!(*stored.pushed());
        assert!(first.is_some());
        assert_eq!(*stored.pushed_at(), first);
    }

    #[tokio::test]
    async fn a_missing_alert_is_not_found() {
        let use_case = MarkAlertPushedUseCase::new(Arc::new(Fakes::new()));

        assert!(matches!(
            use_case.execute(99).await.expect_err("refused"),
            AppError::GlobalAppError(GlobalAppError::NotFound)
        ));
    }
}
