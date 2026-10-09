use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::alerts::{
        app::{AlertFarms, AlertRepository, AppError},
        domain::{Alert, HistoryDays, baghdad_day},
    },
};

pub struct ListFarmAlertsUseCase {
    repository: Arc<dyn AlertRepository>,
    farms: Arc<dyn AlertFarms>,
}

impl ListFarmAlertsUseCase {
    pub fn new(repository: Arc<dyn AlertRepository>, farms: Arc<dyn AlertFarms>) -> Self {
        Self { repository, farms }
    }

    /// Returns the alerts of one of the farmer's own farms, newest day
    /// first. A farm of another phone is answered like one that does not
    /// exist.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        farm_id: i32,
        days: HistoryDays,
    ) -> Result<Vec<Alert>, AppError> {
        if !self
            .farms
            .is_owned_by(farm_id, auth_context.user().phone())
            .await?
        {
            tracing::info!(farm_id, "alerts refused: no such farm for this owner");

            return Err(GlobalAppError::NotFound.into());
        }

        let first_day = days.first_day(baghdad_day(Utc::now()));
        let alerts = self.repository.find_by_farms(&[farm_id], first_day).await?;

        tracing::debug!(farm_id, returned = alerts.len(), "farm alerts listed");

        Ok(alerts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alerts::app::testing::{
        Call, FARM_ID, Fakes, OTHER_FARM_ID, OWNER, an_alert, auth_context,
    };

    fn use_case(fakes: &Fakes) -> ListFarmAlertsUseCase {
        ListFarmAlertsUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn returns_the_alerts_of_a_farm_the_farmer_owns() {
        let fakes = Fakes::new()
            .with_stored(an_alert(1, FARM_ID))
            .with_stored(an_alert(2, OTHER_FARM_ID));

        let alerts = use_case(&fakes)
            .execute(&auth_context(), FARM_ID, HistoryDays::default())
            .await
            .expect("alerts");

        assert_eq!(alerts.len(), 1);
        assert_eq!(*alerts[0].id(), Some(1));
        assert_eq!(
            fakes.calls()[0],
            Call::IsOwnedBy {
                farm_id: FARM_ID,
                phone: OWNER.to_string()
            },
            "ownership is checked with the signed-in phone before anything is read"
        );
    }

    #[tokio::test]
    async fn another_farmers_farm_is_not_found_and_nothing_is_read() {
        let fakes = Fakes::new().with_stored(an_alert(2, OTHER_FARM_ID));

        let error = use_case(&fakes)
            .execute(&auth_context(), OTHER_FARM_ID, HistoryDays::default())
            .await
            .expect_err("refused");

        assert!(matches!(
            error,
            AppError::GlobalAppError(GlobalAppError::NotFound)
        ));
        assert_eq!(fakes.calls().len(), 1, "the alerts were never read");
    }
}
