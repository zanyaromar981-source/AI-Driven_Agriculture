use async_trait::async_trait;

use crate::{features::farms::app::AppError, shared::Phone};

/// Whether a phone belongs to a registered farmer. Owned by the farmers
/// feature; staff may register a farm only for a farmer who exists.
#[async_trait]
pub trait FarmerDirectory: Send + Sync + std::fmt::Debug {
    async fn is_registered(&self, phone: &Phone) -> Result<bool, AppError>;
}
