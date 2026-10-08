use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        farmers::app::{AppError, FarmCounter},
        farms::app::FarmRepository,
    },
    shared::Phone,
};

/// Answers the sign-in flow's one question about farms by asking the farms
/// feature through its own repository port.
#[derive(Debug)]
pub struct FarmsFeatureFarmCounter {
    farms: Arc<dyn FarmRepository>,
}

impl FarmsFeatureFarmCounter {
    pub fn new(farms: Arc<dyn FarmRepository>) -> Self {
        Self { farms }
    }
}

#[async_trait]
impl FarmCounter for FarmsFeatureFarmCounter {
    async fn count_for(&self, phone: &Phone) -> Result<u64, AppError> {
        self.farms.count_by_owner(phone).await.map_err(|error| {
            tracing::error!(%error, "counting farms for sign-in failed");

            GlobalAppError::InternalServerError.into()
        })
    }
}
