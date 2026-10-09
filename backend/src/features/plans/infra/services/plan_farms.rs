use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        farms::app::FarmRepository,
        plans::{
            app::{AppError, PlanFarms},
            domain::PlanSite,
        },
    },
    shared::Phone,
};

/// Answers this feature's questions about farms by asking the farms feature
/// through its own repository port, the one the insights coverage uses, so
/// what "belongs" and "exists" mean stays in one place. Only the id and the
/// centroid cross over.
#[derive(Debug)]
pub struct FarmsFeaturePlanFarms {
    farms: Arc<dyn FarmRepository>,
}

impl FarmsFeaturePlanFarms {
    pub fn new(farms: Arc<dyn FarmRepository>) -> Self {
        Self { farms }
    }
}

#[async_trait]
impl PlanFarms for FarmsFeaturePlanFarms {
    async fn is_owned_by(&self, farm_id: i32, phone: &Phone) -> Result<bool, AppError> {
        self.farms
            .find_by_id_and_owner(farm_id, phone)
            .await
            .map(|farm| farm.is_some())
            .map_err(|error| {
                tracing::error!(%error, farm_id, "checking farm ownership for plans failed");

                GlobalAppError::InternalServerError.into()
            })
    }

    async fn exists(&self, farm_id: i32) -> Result<bool, AppError> {
        self.farms.exists(farm_id).await.map_err(|error| {
            tracing::error!(%error, farm_id, "checking that a farm exists for plans failed");

            GlobalAppError::InternalServerError.into()
        })
    }

    async fn all_sites(&self) -> Result<Vec<PlanSite>, AppError> {
        let locations = self.farms.find_all_locations().await.map_err(|error| {
            tracing::error!(%error, "listing farm locations for plans failed");

            AppError::from(GlobalAppError::InternalServerError)
        })?;

        Ok(locations
            .iter()
            .map(|location| {
                let (lat, lon) = *location.centroid();

                PlanSite::rehydrate(*location.id(), lat, lon)
            })
            .collect())
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
        let plan_farms = FarmsFeaturePlanFarms::new(Arc::new(farms.clone()));

        assert!(plan_farms.is_owned_by(7, &phone()).await.expect("answer"));
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
        let plan_farms = FarmsFeaturePlanFarms::new(Arc::new(FakeFarmRepository::new()));

        assert!(!plan_farms.is_owned_by(7, &phone()).await.expect("answer"));
    }

    #[tokio::test]
    async fn a_farm_the_farms_feature_holds_exists_whoever_owns_it() {
        let farms = FakeFarmRepository::holding(a_farm());
        let plan_farms = FarmsFeaturePlanFarms::new(Arc::new(farms.clone()));

        assert!(plan_farms.exists(7).await.expect("answer"));
        assert!(!plan_farms.exists(8).await.expect("answer"));
    }

    #[tokio::test]
    async fn every_farm_becomes_a_site_with_its_centroid_and_no_owner() {
        let farm = a_farm();
        let farms = FakeFarmRepository::holding(farm.clone());
        let plan_farms = FarmsFeaturePlanFarms::new(Arc::new(farms.clone()));

        let sites = plan_farms.all_sites().await.expect("sites");

        let (lat, lon) = farm.outline().centroid();

        assert_eq!(sites, vec![PlanSite::rehydrate(7, lat, lon)]);
        assert_eq!(farms.calls(), vec![RepositoryCall::FindAllLocations]);
    }

    #[tokio::test]
    async fn a_failure_in_the_farms_feature_is_an_error_not_a_no() {
        let plan_farms = FarmsFeaturePlanFarms::new(Arc::new(FakeFarmRepository::failing()));

        assert!(plan_farms.is_owned_by(7, &phone()).await.is_err());
        assert!(plan_farms.exists(7).await.is_err());
        assert!(plan_farms.all_sites().await.is_err());
    }
}
