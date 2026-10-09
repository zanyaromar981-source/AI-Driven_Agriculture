use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        alerts::app::{AlertRepository, DeviceRepository},
        farmers::app::{AppError, FarmerDataRemover},
        farms::app::FarmRepository,
    },
    shared::Phone,
};

/// Removes a farmer's alerts and registered phones by asking the alerts
/// feature through its own repository ports, and the farms feature for
/// which farms the alerts hang on.
#[derive(Debug)]
pub struct AlertsFeatureFarmerDataRemover {
    farms: Arc<dyn FarmRepository>,
    alerts: Arc<dyn AlertRepository>,
    devices: Arc<dyn DeviceRepository>,
}

impl AlertsFeatureFarmerDataRemover {
    pub fn new(
        farms: Arc<dyn FarmRepository>,
        alerts: Arc<dyn AlertRepository>,
        devices: Arc<dyn DeviceRepository>,
    ) -> Self {
        Self {
            farms,
            alerts,
            devices,
        }
    }
}

fn failed(what: &'static str, error: impl std::fmt::Display) -> AppError {
    tracing::error!(%error, what, "removing a farmer's alert data failed");

    GlobalAppError::InternalServerError.into()
}

#[async_trait]
impl FarmerDataRemover for AlertsFeatureFarmerDataRemover {
    async fn remove_all_for(&self, phone: &Phone) -> Result<(), AppError> {
        let farm_ids: Vec<i32> = self
            .farms
            .find_all_by_owner(phone)
            .await
            .map_err(|error| failed("listing farms", error))?
            .iter()
            .map(|farm| *farm.id())
            .collect();

        let alerts_removed = self
            .alerts
            .delete_by_farms(&farm_ids)
            .await
            .map_err(|error| failed("deleting alerts", error))?;

        let devices_removed = self
            .devices
            .delete_all_by_phone(phone)
            .await
            .map_err(|error| failed("deleting devices", error))?;

        tracing::info!(
            farms = farm_ids.len(),
            alerts_removed,
            devices_removed,
            "farmer's alerts and devices removed"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::{
        alerts::app::testing::{
            Call, FARM_ID, Fakes, OTHER_FARM_ID, OWNER as ALERT_OWNER, an_alert,
        },
        farms::app::testing::{FakeFarmRepository, OWNER, a_farm},
    };

    #[tokio::test]
    async fn removes_the_alerts_of_the_farmers_farms_and_their_devices_only() {
        assert_eq!(OWNER, ALERT_OWNER, "both fakes speak of the same farmer");

        let farms = FakeFarmRepository::holding(a_farm());
        let alerts = Fakes::new()
            .with_device("token-a")
            .with_stored(an_alert(1, FARM_ID))
            .with_stored(an_alert(2, OTHER_FARM_ID));
        let remover = AlertsFeatureFarmerDataRemover::new(
            Arc::new(farms),
            Arc::new(alerts.clone()),
            Arc::new(alerts.clone()),
        );

        remover
            .remove_all_for(&Phone::new(OWNER.to_string()).expect("phone"))
            .await
            .expect("removed");

        assert!(alerts.stored(1).is_none());
        assert!(alerts.stored(2).is_some(), "another farmer's alert stays");
        assert!(alerts.devices().is_empty());
        assert_eq!(
            alerts.calls(),
            vec![
                Call::DeleteByFarms {
                    farm_ids: vec![FARM_ID]
                },
                Call::DeleteDevicesByPhone {
                    phone: OWNER.to_string()
                },
            ]
        );
    }

    #[tokio::test]
    async fn a_failure_in_the_farms_feature_is_an_error_and_nothing_is_removed() {
        let alerts = Fakes::new().with_device("token-a");
        let remover = AlertsFeatureFarmerDataRemover::new(
            Arc::new(FakeFarmRepository::failing()),
            Arc::new(alerts.clone()),
            Arc::new(alerts.clone()),
        );

        assert!(
            remover
                .remove_all_for(&Phone::new(OWNER.to_string()).expect("phone"))
                .await
                .is_err()
        );
        assert_eq!(alerts.devices().len(), 1);
    }
}
