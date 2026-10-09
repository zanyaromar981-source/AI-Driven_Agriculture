use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        farmers::app::FarmerRepository,
        farms::app::{AppError, FarmerDirectory},
    },
    shared::Phone,
};

/// Answers whether a phone has a farmer by asking the farmers feature
/// through its own repository port.
#[derive(Debug)]
pub struct FarmersFeatureFarmerDirectory {
    farmers: Arc<dyn FarmerRepository>,
}

impl FarmersFeatureFarmerDirectory {
    pub fn new(farmers: Arc<dyn FarmerRepository>) -> Self {
        Self { farmers }
    }
}

#[async_trait]
impl FarmerDirectory for FarmersFeatureFarmerDirectory {
    async fn is_registered(&self, phone: &Phone) -> Result<bool, AppError> {
        self.farmers
            .find_by_phone(phone)
            .await
            .map(|farmer| farmer.is_some())
            .map_err(|error| {
                tracing::error!(%error, "looking up a farmer for a farm registration failed");

                GlobalAppError::InternalServerError.into()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::app::testing::{Call, Fakes, PHONE, phone};

    #[tokio::test]
    async fn a_phone_the_farmers_feature_knows_is_registered() {
        let farmers = Fakes::new().with_farmer();
        let directory = FarmersFeatureFarmerDirectory::new(Arc::new(farmers.clone()));

        assert!(directory.is_registered(&phone()).await.expect("answer"));
        assert_eq!(
            farmers.calls(),
            vec![Call::FindFarmer {
                phone: PHONE.to_string()
            }]
        );
    }

    #[tokio::test]
    async fn a_phone_the_farmers_feature_does_not_know_is_not_registered() {
        let directory = FarmersFeatureFarmerDirectory::new(Arc::new(Fakes::new()));

        assert!(!directory.is_registered(&phone()).await.expect("answer"));
    }
}
