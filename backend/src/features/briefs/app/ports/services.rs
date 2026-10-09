use async_trait::async_trait;

use crate::{features::briefs::app::AppError, shared::Phone};

/// Whether a farm belongs to a phone. Owned by the farms feature; showing a
/// farm its brief needs only the yes or no.
#[async_trait]
pub trait BriefFarmOwnership: Send + Sync + std::fmt::Debug {
    async fn is_owned_by(&self, farm_id: i32, phone: &Phone) -> Result<bool, AppError>;
}
