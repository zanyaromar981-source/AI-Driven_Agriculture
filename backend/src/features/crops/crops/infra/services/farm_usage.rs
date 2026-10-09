use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        crops::{
            app::{AppError, CropUsage},
            domain::CropCode,
        },
        farms::{app::FarmRepository, domain::Crop},
    },
};

/// Answers whether any farm cell is painted with a crop by asking the farms
/// feature through its own repository port.
#[derive(Debug)]
pub struct FarmsFeatureCropUsage {
    farms: Arc<dyn FarmRepository>,
}

impl FarmsFeatureCropUsage {
    pub fn new(farms: Arc<dyn FarmRepository>) -> Self {
        Self { farms }
    }
}

#[async_trait]
impl CropUsage for FarmsFeatureCropUsage {
    async fn is_used(&self, code: &CropCode) -> Result<bool, AppError> {
        // Failing to ask must stop the delete: an error here is never read
        // as "not used".
        let crop = Crop::new(code.as_str()).map_err(|error| {
            tracing::error!(%error, "a crop code the farms feature cannot read");

            GlobalAppError::InternalServerError
        })?;

        self.farms.is_crop_painted(crop).await.map_err(|error| {
            tracing::error!(%error, "asking the farms feature about a crop failed");

            GlobalAppError::InternalServerError.into()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::{
        crops::app::testing::code,
        farms::app::testing::{FakeFarmRepository, RepositoryCall, a_farm},
    };

    #[tokio::test]
    async fn a_crop_painted_on_a_cell_is_in_use_and_one_painted_nowhere_is_not() {
        // The fixture farm has one cell of wheat.
        let repository = FakeFarmRepository::holding(a_farm());
        let usage = FarmsFeatureCropUsage::new(Arc::new(repository.clone()));

        assert!(usage.is_used(&code("wheat")).await.expect("answer"));
        assert!(!usage.is_used(&code("rice")).await.expect("answer"));
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::IsCropPainted {
                    crop: Crop::of("wheat")
                },
                RepositoryCall::IsCropPainted {
                    crop: Crop::of("rice")
                },
            ]
        );
    }

    #[tokio::test]
    async fn a_failure_to_ask_is_an_error_and_not_a_no() {
        let usage = FarmsFeatureCropUsage::new(Arc::new(FakeFarmRepository::failing()));

        assert!(usage.is_used(&code("wheat")).await.is_err());
    }
}
