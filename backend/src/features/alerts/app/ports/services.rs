use async_trait::async_trait;

use crate::{features::alerts::app::AppError, shared::Phone};

/// What alerts need to know about farms. Owned by the farms feature.
#[async_trait]
pub trait AlertFarms: Send + Sync + std::fmt::Debug {
    async fn is_owned_by(&self, farm_id: i32, phone: &Phone) -> Result<bool, AppError>;

    /// The ids of every farm the phone has.
    async fn ids_owned_by(&self, phone: &Phone) -> Result<Vec<i32>, AppError>;

    /// The phone a farm belongs to. `None` when there is no such farm. For
    /// the push list only: it never leaves the backend.
    async fn owner_of(&self, farm_id: i32) -> Result<Option<Phone>, AppError>;
}
