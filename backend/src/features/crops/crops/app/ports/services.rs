use async_trait::async_trait;

use crate::features::crops::{app::AppError, domain::CropCode};

/// Whether anything stored elsewhere names a crop. One of these stands for
/// each feature that stores crop codes (farm cells; Alwa listings and
/// prices): the crops feature holds no key into their tables, so it asks.
#[async_trait]
pub trait CropUsage: Send + Sync + std::fmt::Debug {
    async fn is_used(&self, code: &CropCode) -> Result<bool, AppError>;
}
