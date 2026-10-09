use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::AuthContext,
    features::alerts::{
        app::{AppError, DeviceRepository},
        domain::{Device, DeviceLanguage, Platform, PushToken},
    },
};

pub struct RegisterDeviceInput {
    pub push_token: PushToken,
    pub platform: Platform,
    pub lang: DeviceLanguage,
    pub red_alerts: bool,
    pub weekly_plan: bool,
}

pub struct RegisterDeviceUseCase {
    devices: Arc<dyn DeviceRepository>,
}

impl RegisterDeviceUseCase {
    pub fn new(devices: Arc<dyn DeviceRepository>) -> Self {
        Self { devices }
    }

    /// Registers a phone for pushes under the signed-in farmer. The app
    /// sends this on every start: a repeat changes nothing but the
    /// settings, and a token last seen under another number moves to this
    /// one, so the previous user of the phone stops getting pushes on it.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        input: RegisterDeviceInput,
    ) -> Result<(), AppError> {
        let device = Device::new(
            input.push_token,
            auth_context.user().phone().clone(),
            input.platform,
            input.lang,
            input.red_alerts,
            input.weekly_plan,
            Utc::now(),
        );

        self.devices.upsert(&device).await?;

        // The token is a secret and stays out of the log.
        tracing::info!(
            platform = %String::from(*device.platform()),
            red_alerts = *device.red_alerts(),
            weekly_plan = *device.weekly_plan(),
            "device registered"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alerts::app::testing::{
        Fakes, OWNER, auth_context, other_auth_context, token,
    };

    const OTHER: &str = "+9647507654321";

    fn input(red_alerts: bool) -> RegisterDeviceInput {
        RegisterDeviceInput {
            push_token: token("token-a"),
            platform: Platform::Android,
            lang: DeviceLanguage::Ku,
            red_alerts,
            weekly_plan: true,
        }
    }

    #[tokio::test]
    async fn registering_the_same_token_twice_keeps_one_device_with_the_last_settings() {
        let fakes = Fakes::new();
        let use_case = RegisterDeviceUseCase::new(Arc::new(fakes.clone()));

        use_case
            .execute(&auth_context(), input(true))
            .await
            .expect("first");
        use_case
            .execute(&auth_context(), input(false))
            .await
            .expect("second");

        let devices = fakes.devices();

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].phone().as_str(), OWNER);
        assert!(!devices[0].red_alerts());
    }

    #[tokio::test]
    async fn a_token_that_signs_in_under_another_number_moves_to_it() {
        let fakes = Fakes::new();
        let use_case = RegisterDeviceUseCase::new(Arc::new(fakes.clone()));

        use_case
            .execute(&auth_context(), input(true))
            .await
            .expect("first");
        use_case
            .execute(&other_auth_context(OTHER), input(true))
            .await
            .expect("second");

        let devices = fakes.devices();

        assert_eq!(devices.len(), 1, "one phone, one row, whoever holds it");
        assert_eq!(devices[0].phone().as_str(), OTHER);
    }
}
