use async_trait::async_trait;

use crate::{
    features::farms::{
        app::AppError,
        domain::{AreaNames, FarmPlace},
    },
    shared::Phone,
};

/// Whether a phone belongs to a registered farmer. Owned by the farmers
/// feature; staff may register a farm only for a farmer who exists.
#[async_trait]
pub trait FarmerDirectory: Send + Sync + std::fmt::Debug {
    async fn is_registered(&self, phone: &Phone) -> Result<bool, AppError>;
}

/// Which governorate, district and sub-district a point lies in. Owned by
/// the zones feature, which holds the shapes.
#[async_trait]
pub trait PlaceLocator: Send + Sync + std::fmt::Debug {
    /// `None` when the point is outside every sub-district.
    async fn locate(&self, lat: f64, lon: f64) -> Result<Option<FarmPlace>, AppError>;
}

/// What the governorates, districts and sub-districts are called. Owned by
/// the zones feature.
#[async_trait]
pub trait AreaDirectory: Send + Sync + std::fmt::Debug {
    async fn names(&self) -> Result<AreaNames, AppError>;
}
