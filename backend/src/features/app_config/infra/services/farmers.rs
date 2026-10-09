use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        app_config::app::{AppError, AppFarmers},
        farmers::app::FarmerRepository,
    },
    shared::Phone,
};

/// Finds which farmer a phone belongs to by asking the farmers feature
/// through its own repository port. Only the id crosses over.
#[derive(Debug)]
pub struct FarmersFeatureAppFarmers {
    farmers: Arc<dyn FarmerRepository>,
}

impl FarmersFeatureAppFarmers {
    pub fn new(farmers: Arc<dyn FarmerRepository>) -> Self {
        Self { farmers }
    }
}

#[async_trait]
impl AppFarmers for FarmersFeatureAppFarmers {
    async fn farmer_id_of(&self, phone: &Phone) -> Result<Option<i32>, AppError> {
        let farmer = self.farmers.find_by_phone(phone).await.map_err(|error| {
            tracing::error!(%error, "looking up a farmer for an app version sighting failed");

            AppError::from(GlobalAppError::InternalServerError)
        })?;

        Ok(farmer.and_then(|farmer| *farmer.id()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::app::testing::{Call, Fakes, PHONE, phone};

    #[tokio::test]
    async fn a_phone_the_farmers_feature_knows_has_an_id() {
        let farmers = Fakes::new().with_farmer();
        let directory = FarmersFeatureAppFarmers::new(Arc::new(farmers.clone()));

        assert_eq!(directory.farmer_id_of(&phone()).await.expect("id"), Some(1));
        assert_eq!(
            farmers.calls(),
            vec![Call::FindFarmer {
                phone: PHONE.to_string()
            }]
        );
    }

    #[tokio::test]
    async fn a_phone_the_farmers_feature_does_not_know_has_none() {
        let directory = FarmersFeatureAppFarmers::new(Arc::new(Fakes::new()));

        assert_eq!(directory.farmer_id_of(&phone()).await.expect("id"), None);
    }

    #[tokio::test]
    async fn a_failure_in_the_farmers_feature_is_an_error_not_a_no() {
        let directory =
            FarmersFeatureAppFarmers::new(Arc::new(Fakes::new().failing_to_read_farmers()));

        assert!(directory.farmer_id_of(&phone()).await.is_err());
    }
}
