use async_trait::async_trait;

use crate::features::outlooks::{
    app::AppError,
    domain::{IssueMonth, OutlookRun, Season, ZoneOutlook, ZoneSlug},
};

#[async_trait]
pub trait OutlookRepository: Send + Sync + std::fmt::Debug {
    /// Returns the latest season any zone has an outlook for.
    async fn find_latest_season(&self) -> Result<Option<Season>, AppError>;

    /// Returns every month a zone outlook was issued in for the season,
    /// oldest first, each month once.
    async fn find_issue_months(&self, season: &Season) -> Result<Vec<IssueMonth>, AppError>;

    /// Returns the outlook of every zone for one issue, in no promised order.
    async fn find_zone_outlooks(
        &self,
        season: &Season,
        issued: &IssueMonth,
    ) -> Result<Vec<ZoneOutlook>, AppError>;

    /// Returns one zone's outlook at every issue of the season, oldest first.
    async fn find_zone_history(
        &self,
        zone_slug: &ZoneSlug,
        season: &Season,
    ) -> Result<Vec<ZoneOutlook>, AppError>;

    async fn find_run(
        &self,
        season: &Season,
        issued: &IssueMonth,
    ) -> Result<Option<OutlookRun>, AppError>;

    /// Stores the outlook for its zone, season and issue month, replacing the
    /// one already there.
    async fn upsert_zone_outlook(&self, outlook: &ZoneOutlook) -> Result<ZoneOutlook, AppError>;

    /// Stores the track record for its season and issue month, replacing the
    /// one already there.
    async fn upsert_run(&self, run: &OutlookRun) -> Result<OutlookRun, AppError>;
}
