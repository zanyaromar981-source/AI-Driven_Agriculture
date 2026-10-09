use async_trait::async_trait;

use crate::{
    features::history::{app::AppError, domain::FarmSite},
    shared::Phone,
};

/// Whether a farm belongs to a phone. Owned by the farms feature; reading a
/// farm's history needs only the yes or no.
#[async_trait]
pub trait HistoryFarmOwnership: Send + Sync + std::fmt::Debug {
    async fn is_owned_by(&self, farm_id: i32, phone: &Phone) -> Result<bool, AppError>;
}

/// Every farm of every farmer, oldest first. Owned by the farms feature; the
/// data job needs it to know where to fetch a past for, and a push or the
/// dashboard to know whether a farm it names is there at all.
#[async_trait]
pub trait HistoryFarmDirectory: Send + Sync + std::fmt::Debug {
    async fn all_sites(&self) -> Result<Vec<FarmSite>, AppError>;

    /// Whether a farm with that id exists, whoever owns it.
    async fn exists(&self, farm_id: i32) -> Result<bool, AppError>;
}
