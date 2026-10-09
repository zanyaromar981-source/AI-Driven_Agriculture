use async_trait::async_trait;
use chrono::NaiveDate;

use crate::{
    app::Pagination,
    features::briefs::{
        app::AppError,
        domain::{BriefScope, DailyBrief, DayRange, FarmZone, ZoneSlug},
    },
};

#[async_trait]
pub trait BriefRepository: Send + Sync + std::fmt::Debug {
    /// Returns the brief of the newest day stored for the scope.
    async fn find_latest(&self, scope: &BriefScope) -> Result<Option<DailyBrief>, AppError>;

    /// Returns the scope's briefs for the days of the range, newest first.
    async fn find_between(
        &self,
        scope: &BriefScope,
        range: &DayRange,
    ) -> Result<Vec<DailyBrief>, AppError>;

    /// Stores the brief as the one for its day and scope, replacing the one
    /// already there if there is one. Returns the stored brief.
    async fn upsert(&self, entity: &DailyBrief) -> Result<DailyBrief, AppError>;

    /// Removes the brief of that day and scope. Removing one that is not
    /// there is not an error.
    async fn delete(&self, day: NaiveDate, scope: &BriefScope) -> Result<(), AppError>;

    /// Returns the district recorded for the farm, if any.
    async fn find_farm_zone(&self, farm_id: i32) -> Result<Option<ZoneSlug>, AppError>;

    /// Records the district of every farm given, replacing what was recorded
    /// for the same farm, all or nothing. Returns how many were recorded.
    async fn upsert_farm_zones(&self, zones: &[FarmZone]) -> Result<u64, AppError>;

    /// Returns one page of stored briefs, newest day first and then by
    /// scope, with how many match in all. `None` means no filter.
    async fn find_page(
        &self,
        scope: Option<&BriefScope>,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        pagination: &Pagination,
    ) -> Result<(Vec<DailyBrief>, u64), AppError>;

    /// Replaces the brief stored for the entity's day and scope. Returns
    /// `None`, having written nothing, when there is none.
    async fn update(&self, entity: &DailyBrief) -> Result<Option<DailyBrief>, AppError>;
}
