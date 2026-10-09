use async_trait::async_trait;

use crate::{
    features::farms::{
        app::AppError,
        domain::{ActiveCrops, AreaNames, FarmPlace},
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

/// Whether the farm totals may be shown to everyone. Owned by the app
/// settings staff edit.
#[async_trait]
pub trait PublicTotalsSwitch: Send + Sync + std::fmt::Debug {
    async fn is_on(&self) -> Result<bool, AppError>;
}

/// Which crops may be painted on new data. Owned by the crops feature,
/// where staff keep the list. That list also holds the other products of
/// the Marketplace (eggs, honey, sheep): they are not crops and are never
/// among the ones returned here.
#[async_trait]
pub trait CropDirectory: Send + Sync + std::fmt::Debug {
    /// The crops switched on right now. A use case asks once per request,
    /// however many cells the request paints.
    async fn active(&self) -> Result<ActiveCrops, AppError>;
}
