use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        crops::app::CropRepository,
        farms::{
            app::{AppError, CropDirectory},
            domain::{ActiveCrops, Crop},
        },
    },
};

/// Reads which crops may be painted on new farms and cells from the
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

        let codes = crops
            .iter()
            .map(|crop| Crop::new(crop.code().as_str()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| failed(&error))?;

        Ok(ActiveCrops::new(codes))
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

        assert!(active.allow([Crop::of("wheat")]).is_ok());
        assert!(
            active.allow([Crop::of("rice")]).is_err(),
            "a crop switched off is not for new data"
        );
        assert!(active.allow([Crop::of("maize")]).is_err());
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindAll { only_active: true }]
        );
    }

    #[tokio::test]
    async fn a_failure_to_read_the_list_is_an_error_and_not_an_empty_list() {
        let directory = CropsFeatureCropDirectory::new(Arc::new(FakeCropRepository::failing()));

        assert!(directory.active().await.is_err());
    }
}
