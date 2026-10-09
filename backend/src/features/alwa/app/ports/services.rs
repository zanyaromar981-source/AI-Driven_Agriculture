use async_trait::async_trait;

use crate::features::alwa::{
    app::AppError,
    domain::{ActiveCrops, GeoPoint, ZoneSlug},
};

/// Which crops a new listing or price may name. Owned by the crops feature,
/// where staff keep the list.
#[async_trait]
pub trait CropDirectory: Send + Sync + std::fmt::Debug {
    /// The crops switched on right now.
    async fn active(&self) -> Result<ActiveCrops, AppError>;
}

/// Which district a point lies in. Owned by the zones feature, which holds
/// the shapes.
#[async_trait]
pub trait ZoneLocator: Send + Sync + std::fmt::Debug {
    /// `None` when the point is outside every district.
    async fn zone_of(&self, point: &GeoPoint) -> Result<Option<ZoneSlug>, AppError>;
}
