use async_trait::async_trait;

use crate::features::alwa::{app::AppError, domain::ActiveCrops};

/// Which crops a new listing or price may name. Owned by the crops feature,
/// where staff keep the list.
#[async_trait]
pub trait CropDirectory: Send + Sync + std::fmt::Debug {
    /// The crops switched on right now.
    async fn active(&self) -> Result<ActiveCrops, AppError>;
}
