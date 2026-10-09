use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::AuthContext,
    features::alerts::{
        app::{AlertFarms, AlertRepository, AppError},
        domain::{Alert, HistoryDays, baghdad_day},
    },
};

pub struct ListMyAlertsUseCase {
    repository: Arc<dyn AlertRepository>,
    farms: Arc<dyn AlertFarms>,
}

impl ListMyAlertsUseCase {
    pub fn new(repository: Arc<dyn AlertRepository>, farms: Arc<dyn AlertFarms>) -> Self {
        Self { repository, farms }
    }

    /// Returns the alerts of every farm the farmer has, newest day first. A
    /// farmer with no farms gets an empty list.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        days: HistoryDays,
    ) -> Result<Vec<Alert>, AppError> {
        let farm_ids = self.farms.ids_owned_by(auth_context.user().phone()).await?;

        if farm_ids.is_empty() {
            return Ok(Vec::new());
        }

        let first_day = days.first_day(baghdad_day(Utc::now()));
        let alerts = self.repository.find_by_farms(&farm_ids, first_day).await?;

        tracing::debug!(
            farms = farm_ids.len(),
            returned = alerts.len(),
            "own alerts listed"
        );

        Ok(alerts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alerts::app::testing::{
        Call, FARM_ID, Fakes, OTHER_FARM_ID, OWNER, an_alert, auth_context, other_auth_context,
    };

    fn use_case(fakes: &Fakes) -> ListMyAlertsUseCase {
        ListMyAlertsUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn returns_only_the_alerts_of_the_farmers_own_farms() {
        let fakes = Fakes::new()
            .with_stored(an_alert(1, FARM_ID))
            .with_stored(an_alert(2, OTHER_FARM_ID));

        let alerts = use_case(&fakes)
            .execute(&auth_context(), HistoryDays::default())
            .await
            .expect("alerts");

        assert_eq!(alerts.len(), 1);
        assert_eq!(*alerts[0].farm_id(), FARM_ID);
        assert_eq!(
            fakes.calls()[0],
            Call::IdsOwnedBy {
                phone: OWNER.to_string()
            }
        );
    }

    #[tokio::test]
    async fn a_farmer_with_no_farms_gets_an_empty_list_without_a_query() {
        let fakes = Fakes::new().with_stored(an_alert(1, FARM_ID));

        let alerts = use_case(&fakes)
            .execute(
                &other_auth_context("+9647509999999"),
                HistoryDays::default(),
            )
            .await
            .expect("alerts");

        assert!(alerts.is_empty());
        assert_eq!(fakes.calls().len(), 1);
    }
}
