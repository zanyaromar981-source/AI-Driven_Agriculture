use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        briefs::app::{AppError, BriefFarmOwnership},
        farms::app::FarmRepository,
    },
    shared::Phone,
};

/// Answers whether a farm belongs to a phone by asking the farms feature
/// through its own repository port, so the rule for what "belongs" means
/// stays in one place.
#[derive(Debug)]
pub struct FarmsFeatureBriefFarmOwnership {
    farms: Arc<dyn FarmRepository>,
}

impl FarmsFeatureBriefFarmOwnership {
    pub fn new(farms: Arc<dyn FarmRepository>) -> Self {
        Self { farms }
    }
}

#[async_trait]
impl BriefFarmOwnership for FarmsFeatureBriefFarmOwnership {
    async fn is_owned_by(&self, farm_id: i32, phone: &Phone) -> Result<bool, AppError> {
        self.farms
            .find_by_id_and_owner(farm_id, phone)
            .await
            .map(|farm| farm.is_some())
            .map_err(|error| {
                tracing::error!(%error, farm_id, "checking farm ownership for briefs failed");

                GlobalAppError::InternalServerError.into()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::app::testing::{FakeFarmRepository, OWNER, RepositoryCall, a_farm};

    fn phone() -> Phone {
        Phone::new(OWNER.to_string()).expect("phone")
    }

    #[tokio::test]
    async fn a_farm_the_farms_feature_finds_for_the_phone_is_owned() {
        let farms = FakeFarmRepository::holding(a_farm());
        let ownership = FarmsFeatureBriefFarmOwnership::new(Arc::new(farms.clone()));

        assert!(ownership.is_owned_by(7, &phone()).await.expect("answer"));
        assert_eq!(
            farms.calls(),
            vec![RepositoryCall::FindByIdAndOwner {
                id: 7,
                owner: OWNER.to_string(),
            }],
            "the owner must be part of the lookup, never the id alone"
        );
    }

    #[tokio::test]
    async fn a_farm_the_farms_feature_does_not_find_is_not_owned() {
        let ownership = FarmsFeatureBriefFarmOwnership::new(Arc::new(FakeFarmRepository::new()));

        assert!(!ownership.is_owned_by(7, &phone()).await.expect("answer"));
    }

    #[tokio::test]
    async fn a_failure_in_the_farms_feature_is_an_error_not_a_no() {
        let ownership =
            FarmsFeatureBriefFarmOwnership::new(Arc::new(FakeFarmRepository::failing()));

        assert!(ownership.is_owned_by(7, &phone()).await.is_err());
    }
}
