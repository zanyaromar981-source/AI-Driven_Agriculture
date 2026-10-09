use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::features::crops::{
    app::AppError,
    domain::{Crop, CropCode, CropDetails},
};

#[async_trait]
pub trait CropRepository: Send + Sync + std::fmt::Debug {
    /// Returns the crops by sort order and then by code: only the ones
    /// switched on, or all of them.
    async fn find_all(&self, only_active: bool) -> Result<Vec<Crop>, AppError>;

    /// The crop with this code, switched on or not.
    async fn find_by_code(&self, code: &CropCode) -> Result<Option<Crop>, AppError>;

    /// Stores the crop unless one already has the code: `None` then, and
    /// nothing is written.
    async fn create(&self, crop: &Crop) -> Result<Option<Crop>, AppError>;

    /// Replaces everything but the code of the crop with this code. `None`
    /// when there is no such crop.
    async fn update(
        &self,
        code: &CropCode,
        details: &CropDetails,
        now: DateTime<Utc>,
    ) -> Result<Option<Crop>, AppError>;

    /// Removes the crop and says whether there was one.
    async fn delete(&self, code: &CropCode) -> Result<bool, AppError>;
}
