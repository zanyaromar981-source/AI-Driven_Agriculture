use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        farms::app::FarmRepository,
        insights::{
            app::{AppError, FarmDirectory},
            domain::FarmSite,
        },
    },
};

/// Lists every farm by asking the farms feature through its own repository
/// port. Only the id, the centroid and the area cross over.
#[derive(Debug)]
pub struct FarmsFeatureFarmDirectory {
    farms: Arc<dyn FarmRepository>,
}

impl FarmsFeatureFarmDirectory {
    pub fn new(farms: Arc<dyn FarmRepository>) -> Self {
        Self { farms }
    }
}

#[async_trait]
impl FarmDirectory for FarmsFeatureFarmDirectory {
    async fn all_sites(&self) -> Result<Vec<FarmSite>, AppError> {
        let locations = self.farms.find_all_locations().await.map_err(|error| {
            tracing::error!(%error, "listing farm locations for insights failed");

            AppError::from(GlobalAppError::InternalServerError)
        })?;

        Ok(locations
            .iter()
            .map(|location| {
                let (lat, lon) = *location.centroid();

                FarmSite::rehydrate(*location.id(), lat, lon, *location.area_dunam())
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::app::testing::{FakeFarmRepository, RepositoryCall, a_farm};

    #[tokio::test]
    async fn every_farm_becomes_a_site_with_its_centroid_and_area() {
        let farm = a_farm();
        let farms = FakeFarmRepository::holding(farm.clone());
        let directory = FarmsFeatureFarmDirectory::new(Arc::new(farms.clone()));

        let sites = directory.all_sites().await.expect("sites");

        let (lat, lon) = farm.outline().centroid();

        assert_eq!(
            sites,
            vec![FarmSite::rehydrate(7, lat, lon, farm.area_dunam())]
        );
        assert_eq!(farms.calls(), vec![RepositoryCall::FindAllLocations]);
    }

    #[tokio::test]
    async fn no_farms_is_an_empty_list() {
        let directory = FarmsFeatureFarmDirectory::new(Arc::new(FakeFarmRepository::new()));

        assert!(directory.all_sites().await.expect("sites").is_empty());
    }

    #[tokio::test]
    async fn a_failure_in_the_farms_feature_surfaces() {
        let directory = FarmsFeatureFarmDirectory::new(Arc::new(FakeFarmRepository::failing()));

        assert!(directory.all_sites().await.is_err());
    }
}
