use std::sync::Arc;

use crate::{
    app::AuthContext,
    features::alerts::{
        app::{AppError, DeviceRepository},
        domain::PushToken,
    },
};

pub struct RemoveDeviceUseCase {
    devices: Arc<dyn DeviceRepository>,
}

impl RemoveDeviceUseCase {
    pub fn new(devices: Arc<dyn DeviceRepository>) -> Self {
        Self { devices }
    }

    /// Stops pushes to one of the farmer's own phones. A token that is not
    /// registered, or that belongs to another farmer, is left alone and
    /// answered the same way, so nobody learns whose a token is.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        push_token: PushToken,
    ) -> Result<(), AppError> {
        self.devices
            .delete(&push_token, auth_context.user().phone())
            .await?;

        tracing::info!("device removed");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alerts::app::testing::{Fakes, auth_context, other_auth_context, token};

    #[tokio::test]
    async fn a_farmer_removes_their_own_token_and_a_repeat_succeeds() {
        let fakes = Fakes::new().with_device("token-a");
        let use_case = RemoveDeviceUseCase::new(Arc::new(fakes.clone()));

        use_case
            .execute(&auth_context(), token("token-a"))
            .await
            .expect("first");
        use_case
            .execute(&auth_context(), token("token-a"))
            .await
            .expect("second");

        assert!(fakes.devices().is_empty());
    }

    #[tokio::test]
    async fn another_farmers_token_is_left_alone_and_answered_the_same() {
        let fakes = Fakes::new().with_device("token-a");
        let use_case = RemoveDeviceUseCase::new(Arc::new(fakes.clone()));

        use_case
            .execute(&other_auth_context("+9647507654321"), token("token-a"))
            .await
            .expect("same answer");

        assert_eq!(fakes.devices().len(), 1);
    }
}
