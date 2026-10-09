use async_trait::async_trait;

use crate::{
    app::Pagination,
    features::outlooks::{
        app::AppError,
        domain::{IssueMonth, OutlookRun, Season, ZoneOutlook, ZoneSlug},
    },
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

    /// Returns one page of the stored zone outlooks, newest issue first, and
    /// how many match in all. A missing filter leaves that side open.
    async fn find_zone_outlooks_page(
        &self,
        season: Option<&Season>,
        issued: Option<&IssueMonth>,
        pagination: &Pagination,
    ) -> Result<(Vec<ZoneOutlook>, u64), AppError>;

    /// Stores the outlook only if its zone has none for that season and
    /// issue. Returns None, with nothing written, when there already was one.
    async fn create_zone_outlook(
        &self,
        outlook: &ZoneOutlook,
    ) -> Result<Option<ZoneOutlook>, AppError>;

    /// Replaces every non-key field of the outlook for that zone, season and
    /// issue. Returns None, with nothing written, when there is none.
    async fn update_zone_outlook(
        &self,
        outlook: &ZoneOutlook,
    ) -> Result<Option<ZoneOutlook>, AppError>;

    /// Removes the outlook. Returns whether there was one to remove.
    async fn delete_zone_outlook(
        &self,
        zone_slug: &ZoneSlug,
        season: &Season,
        issued: &IssueMonth,
    ) -> Result<bool, AppError>;

    /// Returns every stored track record, newest issue first. The list is
    /// short: one per issue.
    async fn find_runs(&self) -> Result<Vec<OutlookRun>, AppError>;

    /// Stores the track record only if its season and issue have none.
    /// Returns None, with nothing written, when there already was one.
    async fn create_run(&self, run: &OutlookRun) -> Result<Option<OutlookRun>, AppError>;

    /// Replaces every non-key field of the track record for that season and
    /// issue. Returns None, with nothing written, when there is none.
    async fn update_run(&self, run: &OutlookRun) -> Result<Option<OutlookRun>, AppError>;

    /// Removes the track record. Returns whether there was one to remove.
    async fn delete_run(&self, season: &Season, issued: &IssueMonth) -> Result<bool, AppError>;
}
