use async_trait::async_trait;

use crate::{features::app_config::app::AppError, shared::Phone};

/// Which farmer a phone belongs to. Owned by the farmers feature; a version
/// sighting keeps the farmer's id, never the phone.
#[async_trait]
pub trait AppFarmers: Send + Sync + std::fmt::Debug {
    async fn farmer_id_of(&self, phone: &Phone) -> Result<Option<i32>, AppError>;
}
