use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::alerts::app::{AlertRepository, AppError},
};

pub struct RemoveAlertUseCase {
    repository: Arc<dyn AlertRepository>,
}

impl RemoveAlertUseCase {
    pub fn new(repository: Arc<dyn AlertRepository>) -> Self {
        Self { repository }
    }

    /// Removes one alert. One that is already gone is a success, so a
    /// repeated delete gets the same answer as the first. The farm is not
    /// looked up, which also lets staff clear an alert left behind by a
    /// deleted farm.
    pub async fn execute(&self, actor: &StaffContext, alert_id: i32) -> Result<(), AppError> {
        let removed = self.repository.delete(alert_id).await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            alert_id,
            removed,
            "alert removed by staff"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alerts::app::testing::{Call, FARM_ID, Fakes, an_alert, staff_context};

    #[tokio::test]
    async fn removes_the_alert_and_a_repeat_succeeds() {
        let fakes = Fakes::new().with_stored(an_alert(1, FARM_ID));
        let use_case = RemoveAlertUseCase::new(Arc::new(fakes.clone()));

        use_case.execute(&staff_context(), 1).await.expect("first");
        use_case.execute(&staff_context(), 1).await.expect("second");

        assert!(fakes.stored(1).is_none());
        assert_eq!(
            fakes.calls(),
            vec![Call::Delete { id: 1 }, Call::Delete { id: 1 }]
        );
    }
}
