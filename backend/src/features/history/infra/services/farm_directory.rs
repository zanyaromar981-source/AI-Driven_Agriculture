use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        farms::app::FarmRepository,
        history::{
            app::{AppError, HistoryFarmDirectory},
            domain::FarmSite,
        },
    },
};

/// Lists every farm by asking the farms feature through its own repository
/// port. Only the id, the centroid and the day the farm was registered
/// cross over.
#[derive(Debug)]
pub struct FarmsFeatureHistoryFarmDirectory {
    farms: Arc<dyn FarmRepository>,
}

impl FarmsFeatureHistoryFarmDirectory {
    pub fn new(farms: Arc<dyn FarmRepository>) -> Self {
        Self { farms }
    }
}

#[async_trait]
impl HistoryFarmDirectory for FarmsFeatureHistoryFarmDirectory {
    async fn all_sites(&self) -> Result<Vec<FarmSite>, AppError> {
        let locations = self
            .farms
            .find_all_locations_with_created_at()
            .await
            .map_err(|error| {
                tracing::error!(%error, "listing farm locations for history failed");

                AppError::from(GlobalAppError::InternalServerError)
            })?;

        Ok(locations
            .iter()
            .map(|(location, created_at)| {
                let (lat, lon) = *location.centroid();

                FarmSite::rehydrate(*location.id(), lat, lon, *created_at)
            })
            .collect())
    }

    async fn exists(&self, farm_id: i32) -> Result<bool, AppError> {
        self.farms.exists(farm_id).await.map_err(|error| {
            tracing::error!(%error, farm_id, "checking that a farm exists for history failed");

            GlobalAppError::InternalServerError.into()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::app::testing::{FakeFarmRepository, RepositoryCall, a_farm};

    #[tokio::test]
    async fn every_farm_becomes_a_site_with_its_centroid_and_the_day_it_was_registered() {
        let farm = a_farm();
        let farms = FakeFarmRepository::holding(farm.clone());
        let directory = FarmsFeatureHistoryFarmDirectory::new(Arc::new(farms.clone()));

        let sites = directory.all_sites().await.expect("sites");

        let (lat, lon) = farm.outline().centroid();

        assert_eq!(
            sites,
            vec![FarmSite::rehydrate(7, lat, lon, *farm.created_at())]
        );
        assert_eq!(
            farms.calls(),
            vec![RepositoryCall::FindAllLocationsWithCreatedAt]
        );
    }

    #[tokio::test]
    async fn no_farms_is_an_empty_list() {
        let directory = FarmsFeatureHistoryFarmDirectory::new(Arc::new(FakeFarmRepository::new()));

        assert!(directory.all_sites().await.expect("sites").is_empty());
    }

    #[tokio::test]
    async fn a_failure_in_the_farms_feature_surfaces() {
        let directory =
            FarmsFeatureHistoryFarmDirectory::new(Arc::new(FakeFarmRepository::failing()));

        assert!(directory.all_sites().await.is_err());
    }

    #[tokio::test]
    async fn a_farm_the_farms_feature_holds_exists_whoever_owns_it() {
        let farms = FakeFarmRepository::holding(a_farm());
        let directory = FarmsFeatureHistoryFarmDirectory::new(Arc::new(farms.clone()));

        assert!(directory.exists(7).await.expect("answer"));
        assert!(!directory.exists(8).await.expect("answer"));
        assert_eq!(
            farms.calls(),
            vec![
                RepositoryCall::Exists { id: 7 },
                RepositoryCall::Exists { id: 8 }
            ],
            "only the id crosses over: no owner is asked for or returned"
        );
    }

    #[tokio::test]
    async fn a_failure_while_checking_is_an_error_not_a_no() {
        let directory =
            FarmsFeatureHistoryFarmDirectory::new(Arc::new(FakeFarmRepository::failing()));

        assert!(directory.exists(7).await.is_err());
    }
}
