use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, StaffContext},
    features::alerts::{
        app::{AlertFarms, AlertRepository, AppError},
        domain::Alert,
    },
};

pub struct ViewStoredFarmAlertsUseCase {
    repository: Arc<dyn AlertRepository>,
    farms: Arc<dyn AlertFarms>,
}

impl ViewStoredFarmAlertsUseCase {
    pub fn new(repository: Arc<dyn AlertRepository>, farms: Arc<dyn AlertFarms>) -> Self {
        Self { repository, farms }
    }

    /// Returns every alert of any farm, newest day first, for staff.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        farm_id: i32,
    ) -> Result<Vec<Alert>, AppError> {
        if self.farms.owner_of(farm_id).await?.is_none() {
            return Err(GlobalAppError::NotFound.into());
        }

        let alerts = self.repository.find_all_by_farm(farm_id).await?;

        tracing::debug!(
            staff_id = *actor.staff_id(),
            farm_id,
            returned = alerts.len(),
            "farm alerts viewed by staff"
        );

        Ok(alerts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alerts::app::testing::{
        FARM_ID, Fakes, OTHER_FARM_ID, an_alert, staff_context,
    };

    fn use_case(fakes: &Fakes) -> ViewStoredFarmAlertsUseCase {
        ViewStoredFarmAlertsUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn staff_see_the_alerts_of_any_farm() {
        let fakes = Fakes::new()
            .with_stored(an_alert(1, FARM_ID))
            .with_stored(an_alert(2, OTHER_FARM_ID));

        let alerts = use_case(&fakes)
            .execute(&staff_context(), OTHER_FARM_ID)
            .await
            .expect("alerts");

        assert_eq!(alerts.len(), 1);
        assert_eq!(*alerts[0].id(), Some(2));
    }

    #[tokio::test]
    async fn a_farm_that_does_not_exist_is_not_found() {
        assert!(matches!(
            use_case(&Fakes::new())
                .execute(&staff_context(), 999)
                .await
                .expect_err("refused"),
            AppError::GlobalAppError(GlobalAppError::NotFound)
        ));
    }
}
