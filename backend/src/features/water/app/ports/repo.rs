use async_trait::async_trait;

use crate::features::water::{
    app::AppError,
    domain::{Season, WaterPlanEntry, ZoneSlug},
};

#[async_trait]
pub trait WaterPlanRepository: Send + Sync + std::fmt::Debug {
    /// Returns the latest season that has at least one plan entry.
    async fn find_latest_season(&self) -> Result<Option<Season>, AppError>;

    /// Returns every entry of the season's plan, in no promised order: the
    /// ranking is the domain's to make. The list is short, one per zone.
    async fn find_by_season(&self, season: &Season) -> Result<Vec<WaterPlanEntry>, AppError>;

    /// Stores the entry for its season and zone, replacing the one already
    /// there.
    async fn upsert(&self, entry: &WaterPlanEntry) -> Result<WaterPlanEntry, AppError>;

    /// Removes the entry. Returns whether there was one to remove.
    async fn delete(&self, season: &Season, zone_slug: &ZoneSlug) -> Result<bool, AppError>;
}
