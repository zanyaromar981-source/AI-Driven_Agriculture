use async_trait::async_trait;

use crate::{
    features::plans::{app::AppError, domain::PlanSite},
    shared::Phone,
};

/// What this feature needs to know about farms. Owned by the farms feature:
/// whether a farm belongs to a phone, whether it exists at all, and where
/// every farm is.
#[async_trait]
pub trait PlanFarms: Send + Sync + std::fmt::Debug {
    async fn is_owned_by(&self, farm_id: i32, phone: &Phone) -> Result<bool, AppError>;

    /// Whether a farm with that id exists, whoever owns it.
    async fn exists(&self, farm_id: i32) -> Result<bool, AppError>;

    /// Every farm of every farmer, oldest first, without the owners.
    async fn all_sites(&self) -> Result<Vec<PlanSite>, AppError>;
}
