use std::sync::Arc;

use crate::features::alerts::{
    app::{AppError, DeviceRepository},
    domain::PushToken,
};

pub struct RemoveDeadDeviceUseCase {
    devices: Arc<dyn DeviceRepository>,
}

impl RemoveDeadDeviceUseCase {
    pub fn new(devices: Arc<dyn DeviceRepository>) -> Self {
        Self { devices }
    }

    /// Forgets a token the push service says is dead, whoever it belongs
    /// to. For the push sender only. One that is already gone is a success.
    pub async fn execute(&self, push_token: PushToken) -> Result<(), AppError> {
        self.devices.delete_by_token(&push_token).await?;

        tracing::info!("dead device removed by the push sender");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alerts::app::testing::{Call, Fakes, token};

    #[tokio::test]
    async fn removes_the_token_whoever_owns_it_and_a_repeat_succeeds() {
        let fakes = Fakes::new().with_device("token-a").with_device("token-b");
        let use_case = RemoveDeadDeviceUseCase::new(Arc::new(fakes.clone()));

        use_case.execute(token("token-a")).await.expect("first");
        use_case.execute(token("token-a")).await.expect("second");

        let left = fakes.devices();

        assert_eq!(left.len(), 1);
        assert_eq!(left[0].push_token().as_str(), "token-b");
        assert_eq!(
            fakes.calls(),
            vec![Call::DeleteDeviceByToken, Call::DeleteDeviceByToken],
            "the token itself is never recorded"
        );
    }
}
