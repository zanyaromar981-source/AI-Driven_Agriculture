use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        app_config::app::AppConfigRepository,
        farms::app::{AppError, PublicTotalsSwitch},
    },
};

/// Reads the `public_farm_totals` switch from the app settings feature
/// through its own repository port, on every call, so switching it off
/// takes effect at once.
#[derive(Debug)]
pub struct AppConfigPublicTotalsSwitch {
    config: Arc<dyn AppConfigRepository>,
}

impl AppConfigPublicTotalsSwitch {
    pub fn new(config: Arc<dyn AppConfigRepository>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl PublicTotalsSwitch for AppConfigPublicTotalsSwitch {
    async fn is_on(&self) -> Result<bool, AppError> {
        let config = self.config.find().await.map_err(|error| {
            tracing::error!(%error, "reading the public farm totals switch failed");

            AppError::from(GlobalAppError::InternalServerError)
        })?;

        Ok(config.settings().public_farm_totals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::app_config::app::testing::Fakes;

    #[tokio::test]
    async fn the_switch_is_what_the_app_settings_hold() {
        for on in [true, false] {
            let switch = AppConfigPublicTotalsSwitch::new(Arc::new(
                Fakes::new().with_public_farm_totals(on),
            ));

            assert_eq!(switch.is_on().await.expect("answer"), on);
        }
    }

    #[tokio::test]
    async fn a_failure_in_the_app_settings_is_an_error_not_a_yes_or_a_no() {
        let switch = AppConfigPublicTotalsSwitch::new(Arc::new(Fakes::new().failing()));

        assert!(switch.is_on().await.is_err());
    }
}
