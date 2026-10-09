use std::{collections::HashSet, sync::Arc};

use chrono::{Duration, Utc};

use crate::features::alerts::{
    app::{AlertFarms, AlertRepository, AppError, DeviceRepository},
    domain::{PendingPush, baghdad_day, baghdad_day_start, one_per_farm},
};

pub struct ListUnpushedAlertsUseCase {
    repository: Arc<dyn AlertRepository>,
    devices: Arc<dyn DeviceRepository>,
    farms: Arc<dyn AlertFarms>,
}

impl ListUnpushedAlertsUseCase {
    pub fn new(
        repository: Arc<dyn AlertRepository>,
        devices: Arc<dyn DeviceRepository>,
        farms: Arc<dyn AlertFarms>,
    ) -> Self {
        Self {
            repository,
            devices,
            farms,
        }
    }

    /// Returns the alarms a push sender may send now, each with the farm
    /// owner's phones that want red alerts (none is still listed). At most one per
    /// farm, and none for a farm that already had a push today (the Iraqi
    /// day). An alarm of a farm that no longer exists is left out.
    pub async fn execute(&self) -> Result<Vec<PendingPush>, AppError> {
        let now = Utc::now();
        let today = baghdad_day(now);

        // The domain rule decides what is due; the day filter only keeps
        // the query from reading every old alarm.
        let candidates = self
            .repository
            .find_unpushed_alarms(today - Duration::days(1))
            .await?;

        let pushed_today: HashSet<i32> = self
            .repository
            .find_farms_pushed_since(baghdad_day_start(now))
            .await?
            .into_iter()
            .collect();

        let mut pending = Vec::new();

        for alert in one_per_farm(candidates, &pushed_today, today) {
            let Some(owner) = self.farms.owner_of(*alert.farm_id()).await? else {
                continue;
            };

            let devices = self.devices.find_alert_devices(&owner).await?;

            pending.push(PendingPush::new(alert, devices));
        }

        tracing::debug!(returned = pending.len(), "unpushed alarms listed");

        Ok(pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alerts::{
        app::testing::{FARM_ID, Fakes, OTHER_FARM_ID, an_alarm, an_alert},
        domain::AlertLevel,
    };

    fn use_case(fakes: &Fakes) -> ListUnpushedAlertsUseCase {
        ListUnpushedAlertsUseCase::new(
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
        )
    }

    #[tokio::test]
    async fn an_alarm_is_listed_with_the_owners_tokens_and_a_watch_is_not() {
        let fakes = Fakes::new()
            .with_device("token-a")
            .with_stored(an_alarm(1, FARM_ID))
            .with_stored(an_alert(2, FARM_ID));

        let pending = use_case(&fakes).execute().await.expect("pending");

        assert_eq!(pending.len(), 1);
        assert_eq!(*pending[0].alert().level(), AlertLevel::Alarm);
        assert_eq!(pending[0].devices().len(), 1);
        assert_eq!(pending[0].devices()[0].push_token().as_str(), "token-a");
    }

    #[tokio::test]
    async fn a_farm_that_had_a_push_today_is_left_out() {
        let fakes = Fakes::new()
            .with_stored(an_alarm(1, FARM_ID))
            .with_stored(an_alarm(2, FARM_ID))
            .with_stored(an_alarm(3, OTHER_FARM_ID));

        fakes.push(1);

        let pending = use_case(&fakes).execute().await.expect("pending");
        let farms: Vec<i32> = pending.iter().map(|push| *push.alert().farm_id()).collect();

        assert_eq!(
            farms,
            vec![OTHER_FARM_ID],
            "one push per farm per day: farm 7 waits for tomorrow"
        );
    }

    #[tokio::test]
    async fn an_alarm_whose_owner_has_no_phone_is_still_listed_with_no_devices() {
        let fakes = Fakes::new().with_stored(an_alarm(1, FARM_ID));

        let pending = use_case(&fakes).execute().await.expect("pending");

        assert_eq!(pending.len(), 1);
        assert!(pending[0].devices().is_empty());
    }

    #[tokio::test]
    async fn an_alarm_of_a_farm_that_is_gone_is_left_out() {
        let fakes = Fakes::new().with_stored(an_alarm(1, 999));

        assert!(
            use_case(&fakes)
                .execute()
                .await
                .expect("pending")
                .is_empty()
        );
    }
}
