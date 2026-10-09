use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        alwa::{app::AlwaRepository, domain::Crop},
        crops::{
            app::{AppError, CropUsage},
            domain::CropCode,
        },
    },
};

/// Answers whether any Alwa listing or price names a crop by asking the
/// alwa feature through its own repository port.
#[derive(Debug)]
pub struct AlwaFeatureCropUsage {
    alwa: Arc<dyn AlwaRepository>,
}

impl AlwaFeatureCropUsage {
    pub fn new(alwa: Arc<dyn AlwaRepository>) -> Self {
        Self { alwa }
    }
}

#[async_trait]
impl CropUsage for AlwaFeatureCropUsage {
    async fn is_used(&self, code: &CropCode) -> Result<bool, AppError> {
        // Failing to ask must stop the delete: an error here is never read
        // as "not used".
        let crop = Crop::new(code.as_str()).map_err(|error| {
            tracing::error!(%error, "a crop code the alwa feature cannot read");

            GlobalAppError::InternalServerError
        })?;

        self.alwa.is_crop_traded(crop).await.map_err(|error| {
            tracing::error!(%error, "asking the alwa feature about a crop failed");

            GlobalAppError::InternalServerError.into()
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;
    use crate::features::{
        alwa::app::testing::{FakeAlwaRepository, MARKET_ID, a_price, an_open_listing},
        crops::app::testing::code,
    };

    #[tokio::test]
    async fn a_crop_a_listing_names_is_in_use() {
        // The fixture listing sells tomato.
        let usage = AlwaFeatureCropUsage::new(Arc::new(
            FakeAlwaRepository::new().with_listing(an_open_listing(1)),
        ));

        assert!(usage.is_used(&code("tomato")).await.expect("answer"));
        assert!(!usage.is_used(&code("rice")).await.expect("answer"));
    }

    #[tokio::test]
    async fn a_crop_a_price_names_is_in_use() {
        let day = NaiveDate::from_ymd_opt(2026, 10, 8).expect("day");
        let usage =
            AlwaFeatureCropUsage::new(Arc::new(FakeAlwaRepository::new().with_price(a_price(
                MARKET_ID,
                Crop::of("okra"),
                day,
                900,
                false,
            ))));

        assert!(usage.is_used(&code("okra")).await.expect("answer"));
        assert!(!usage.is_used(&code("tomato")).await.expect("answer"));
    }

    #[tokio::test]
    async fn a_failure_to_ask_is_an_error_and_not_a_no() {
        let usage = AlwaFeatureCropUsage::new(Arc::new(FakeAlwaRepository::failing()));

        assert!(usage.is_used(&code("tomato")).await.is_err());
    }
}
