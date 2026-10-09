use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        alwa::{
            app::{AppError, CropDirectory},
            domain::{ActiveCrops, AlwaError, Crop, Product, ProductGroup, Unit},
        },
        crops::app::CropRepository,
    },
};

/// Reads which crops may be named by new listings and prices from the
/// crops feature through its own repository port, on every call, so a crop
/// staff switch on or off counts from the next request.
#[derive(Debug)]
pub struct CropsFeatureCropDirectory {
    crops: Arc<dyn CropRepository>,
}

impl CropsFeatureCropDirectory {
    pub fn new(crops: Arc<dyn CropRepository>) -> Self {
        Self { crops }
    }
}

#[async_trait]
impl CropDirectory for CropsFeatureCropDirectory {
    async fn active(&self) -> Result<ActiveCrops, AppError> {
        // Not knowing the list must refuse the write, never wave it through.
        let failed = |error: &dyn std::fmt::Display| {
            tracing::error!(%error, "reading the active crops failed");

            AppError::from(GlobalAppError::InternalServerError)
        };

        let crops = self
            .crops
            .find_all(true)
            .await
            .map_err(|error| failed(&error))?;

        // The two features name groups and units with the same words; the
        // word is what crosses, so neither depends on the other's enum.
        let products = crops
            .iter()
            .map(|crop| {
                let details = crop.details();

                Ok::<_, AlwaError>(Product::new(
                    Crop::new(crop.code().as_str())?,
                    ProductGroup::try_from(String::from(details.group).as_str())?,
                    Unit::try_from(String::from(details.unit).as_str())?,
                ))
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| failed(&error))?;

        Ok(ActiveCrops::new(products))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::crops::app::testing::{FakeCropRepository, RepositoryCall, a_crop};

    #[tokio::test]
    async fn only_the_crops_switched_on_are_allowed_and_the_list_is_read_once() {
        let repository =
            FakeCropRepository::holding(vec![a_crop("wheat", 10, true), a_crop("rice", 20, false)]);
        let directory = CropsFeatureCropDirectory::new(Arc::new(repository.clone()));

        let active = directory.active().await.expect("active crops");

        assert!(active.allow(Crop::of("wheat")).is_ok());
        assert!(
            active.allow(Crop::of("rice")).is_err(),
            "a crop switched off is not for new data"
        );
        assert!(active.allow(Crop::of("maize")).is_err());
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindAll { only_active: true }]
        );
    }

    #[tokio::test]
    async fn a_product_that_is_not_a_crop_is_allowed_and_keeps_its_group_and_unit() {
        use crate::features::crops::{app::testing::a_product, domain};

        let directory =
            CropsFeatureCropDirectory::new(Arc::new(FakeCropRepository::holding(vec![
                a_crop("wheat", 10, true),
                a_product(
                    "eggs",
                    230,
                    domain::ProductGroup::FishMeatEggs,
                    domain::ProductUnit::Tray30,
                ),
                a_product(
                    "cow",
                    300,
                    domain::ProductGroup::Animals,
                    domain::ProductUnit::Head,
                ),
            ])));

        let active = directory.active().await.expect("active crops");

        assert_eq!(
            active.allow(Crop::of("wheat")).expect("wheat"),
            Product::crop("wheat")
        );
        assert_eq!(
            active.allow(Crop::of("eggs")).expect("eggs"),
            Product::of("eggs", ProductGroup::FishMeatEggs, Unit::Tray30)
        );
        assert_eq!(
            active.allow(Crop::of("cow")).expect("cow"),
            Product::of("cow", ProductGroup::Animals, Unit::Head)
        );
    }

    #[test]
    fn every_group_and_unit_of_the_crops_feature_has_its_word_here() {
        use crate::features::crops::domain;

        for group in domain::ProductGroup::ALL {
            assert!(ProductGroup::try_from(String::from(group).as_str()).is_ok());
        }

        for unit in domain::ProductUnit::ALL {
            assert!(Unit::try_from(String::from(unit).as_str()).is_ok());
        }
    }

    #[tokio::test]
    async fn a_failure_to_read_the_list_is_an_error_and_not_an_empty_list() {
        let directory = CropsFeatureCropDirectory::new(Arc::new(FakeCropRepository::failing()));

        assert!(directory.active().await.is_err());
    }
}
