use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        alerts::app::{AlertFarms, AppError},
        farms::app::{AppError as FarmsAppError, FarmRepository},
    },
    shared::Phone,
};

/// Answers what alerts need to know about farms by asking the farms feature
/// through its own repository port, so the rule for what "belongs" means
/// stays in one place.
#[derive(Debug)]
pub struct FarmsFeatureAlertFarms {
    farms: Arc<dyn FarmRepository>,
}

impl FarmsFeatureAlertFarms {
    pub fn new(farms: Arc<dyn FarmRepository>) -> Self {
        Self { farms }
    }
}

fn farms_error(error: FarmsAppError) -> AppError {
    tracing::error!(%error, "asking the farms feature for alerts failed");

    GlobalAppError::InternalServerError.into()
}

#[async_trait]
impl AlertFarms for FarmsFeatureAlertFarms {
    async fn is_owned_by(&self, farm_id: i32, phone: &Phone) -> Result<bool, AppError> {
        self.farms
            .find_by_id_and_owner(farm_id, phone)
            .await
            .map(|farm| farm.is_some())
            .map_err(farms_error)
    }

    async fn ids_owned_by(&self, phone: &Phone) -> Result<Vec<i32>, AppError> {
        Ok(self
            .farms
            .find_all_by_owner(phone)
            .await
            .map_err(farms_error)?
            .iter()
            .map(|farm| *farm.id())
            .collect())
    }

    async fn owner_of(&self, farm_id: i32) -> Result<Option<Phone>, AppError> {
        Ok(self
            .farms
            .find_by_id(farm_id)
            .await
            .map_err(farms_error)?
            .map(|farm| farm.owner().clone()))
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
    async fn a_farm_is_owned_only_when_the_farms_feature_finds_it_for_the_phone() {
        let farms = FakeFarmRepository::holding(a_farm());
        let adapter = FarmsFeatureAlertFarms::new(Arc::new(farms.clone()));

        let nothing = FarmsFeatureAlertFarms::new(Arc::new(FakeFarmRepository::new()));

        assert!(adapter.is_owned_by(7, &phone()).await.expect("answer"));
        assert!(!nothing.is_owned_by(7, &phone()).await.expect("answer"));
        assert_eq!(
            farms.calls()[0],
            RepositoryCall::FindByIdAndOwner {
                id: 7,
                owner: OWNER.to_string(),
            },
            "the owner must be part of the lookup, never the id alone"
        );
    }

    #[tokio::test]
    async fn the_owner_of_a_farm_is_its_phone_and_a_missing_farm_has_none() {
        let adapter = FarmsFeatureAlertFarms::new(Arc::new(FakeFarmRepository::holding(a_farm())));

        let nothing = FarmsFeatureAlertFarms::new(Arc::new(FakeFarmRepository::new()));

        assert_eq!(adapter.owner_of(7).await.expect("answer"), Some(phone()));
        assert_eq!(nothing.owner_of(7).await.expect("answer"), None);
    }

    #[tokio::test]
    async fn a_failure_in_the_farms_feature_is_an_error_not_a_no() {
        let adapter = FarmsFeatureAlertFarms::new(Arc::new(FakeFarmRepository::failing()));

        assert!(adapter.is_owned_by(7, &phone()).await.is_err());
        assert!(adapter.ids_owned_by(&phone()).await.is_err());
        assert!(adapter.owner_of(7).await.is_err());
    }
}
