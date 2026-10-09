use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::{
    app::Pagination,
    features::outlooks::{
        app::{
            AppError,
            use_cases::{
                ListZoneOutlooksInput, RecordOutlookRunInput, RecordZoneOutlookInput,
                ViewSeasonOutlookInput, ViewZoneOutlookInput,
            },
        },
        domain::{
            self, Confidence, IssueMonth, OutlookCounts, OutlookRun, Reason, RunMethod, Season,
            SeasonOutlook, ZoneOutlook, ZoneSlug,
        },
    },
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Outlook {
    Good,
    Normal,
    Bad,
}

impl From<Outlook> for domain::Outlook {
    fn from(value: Outlook) -> Self {
        match value {
            Outlook::Good => domain::Outlook::Good,
            Outlook::Normal => domain::Outlook::Normal,
            Outlook::Bad => domain::Outlook::Bad,
        }
    }
}

impl From<domain::Outlook> for Outlook {
    fn from(value: domain::Outlook) -> Self {
        match value {
            domain::Outlook::Good => Outlook::Good,
            domain::Outlook::Normal => Outlook::Normal,
            domain::Outlook::Bad => Outlook::Bad,
        }
    }
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SeasonOutlookQuery {
    /// Season like `2026-27`. Defaults to the latest season with an outlook.
    pub season: Option<String>,
    /// Issue month, `YYYY-MM`. Defaults to the latest issue of the season.
    pub issued: Option<String>,
}

impl SeasonOutlookQuery {
    pub fn into_input(self) -> Result<ViewSeasonOutlookInput, AppError> {
        Ok(ViewSeasonOutlookInput {
            season: self.season.map(Season::new).transpose()?,
            issued: self
                .issued
                .map(|issued| IssueMonth::new(&issued))
                .transpose()?,
        })
    }
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ZoneOutlookQuery {
    /// Season like `2026-27`. Defaults to the latest season with an outlook.
    pub season: Option<String>,
}

impl ZoneOutlookQuery {
    pub fn into_input(self, zone_slug: String) -> Result<ViewZoneOutlookInput, AppError> {
        Ok(ViewZoneOutlookInput {
            zone_slug: ZoneSlug::new(zone_slug)?,
            season: self.season.map(Season::new).transpose()?,
        })
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct RecordZoneOutlookParams {
    pub outlook: Outlook,
    /// 0 to 100.
    pub confidence_pct: f64,
    /// One short sentence in English, 200 characters max.
    pub reason_en: Option<String>,
    /// The same sentence in Sorani, 200 characters max.
    pub reason_ku: Option<String>,
}

impl RecordZoneOutlookParams {
    pub fn into_input(
        self,
        season: String,
        issued: String,
        zone_slug: String,
    ) -> Result<RecordZoneOutlookInput, AppError> {
        Ok(RecordZoneOutlookInput {
            zone_slug: ZoneSlug::new(zone_slug)?,
            season: Season::new(season)?,
            issued: IssueMonth::new(&issued)?,
            outlook: self.outlook.into(),
            confidence: Confidence::new(self.confidence_pct)?,
            reason_en: Reason::optional(self.reason_en)?,
            reason_ku: Reason::optional(self.reason_ku)?,
        })
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct RecordOutlookRunParams {
    /// Past seasons the method was tested on.
    pub seasons_tested: i32,
    /// How many of them it called right. Never more than `seasons_tested`.
    pub seasons_right: i32,
    pub method: String,
}

impl RecordOutlookRunParams {
    pub fn into_input(
        self,
        season: String,
        issued: String,
    ) -> Result<RecordOutlookRunInput, AppError> {
        Ok(RecordOutlookRunInput {
            season: Season::new(season)?,
            issued: IssueMonth::new(&issued)?,
            seasons_tested: self.seasons_tested,
            seasons_right: self.seasons_right,
            method: RunMethod::new(self.method)?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct OutlookCountsResponse {
    pub good: usize,
    pub normal: usize,
    pub bad: usize,
}

impl From<&OutlookCounts> for OutlookCountsResponse {
    fn from(counts: &OutlookCounts) -> Self {
        Self {
            good: counts.good,
            normal: counts.normal,
            bad: counts.bad,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ZoneOutlookResponse {
    pub zone_slug: String,
    pub outlook: Outlook,
    pub confidence_pct: f64,
    pub reason_en: Option<String>,
    pub reason_ku: Option<String>,
}

impl From<&ZoneOutlook> for ZoneOutlookResponse {
    fn from(outlook: &ZoneOutlook) -> Self {
        Self {
            zone_slug: outlook.zone_slug().into(),
            outlook: (*outlook.outlook()).into(),
            confidence_pct: outlook.confidence().value(),
            reason_en: outlook.reason_en().as_ref().map(Into::into),
            reason_ku: outlook.reason_ku().as_ref().map(Into::into),
        }
    }
}

/// How often the method was right on past seasons, so the outlook can be
/// read with the trust it has earned.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct TrackRecordResponse {
    pub seasons_tested: i32,
    pub seasons_right: i32,
    pub method: String,
}

impl From<&OutlookRun> for TrackRecordResponse {
    fn from(run: &OutlookRun) -> Self {
        Self {
            seasons_tested: *run.seasons_tested(),
            seasons_right: *run.seasons_right(),
            method: run.method().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct SeasonOutlookResponse {
    pub season: String,
    /// The issue month shown, `YYYY-MM`.
    pub issued: String,
    pub counts: OutlookCountsResponse,
    /// Bad first, then by confidence, highest first.
    pub zones: Vec<ZoneOutlookResponse>,
    /// Null when the job sent no track record for this issue.
    pub track_record: Option<TrackRecordResponse>,
    /// Every issue month stored for the season, `YYYY-MM`, oldest first.
    pub issues: Vec<String>,
}

impl From<&SeasonOutlook> for SeasonOutlookResponse {
    fn from(board: &SeasonOutlook) -> Self {
        Self {
            season: board.season().into(),
            issued: board.issued().into(),
            counts: board.counts().into(),
            zones: board.zones().iter().map(Into::into).collect(),
            track_record: board.track_record().as_ref().map(Into::into),
            issues: board.issues().iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ZoneIssueResponse {
    /// `YYYY-MM`.
    pub issued: String,
    pub outlook: Outlook,
    pub confidence_pct: f64,
    pub reason_en: Option<String>,
    pub reason_ku: Option<String>,
}

impl From<&ZoneOutlook> for ZoneIssueResponse {
    fn from(outlook: &ZoneOutlook) -> Self {
        Self {
            issued: outlook.issued().into(),
            outlook: (*outlook.outlook()).into(),
            confidence_pct: outlook.confidence().value(),
            reason_en: outlook.reason_en().as_ref().map(Into::into),
            reason_ku: outlook.reason_ku().as_ref().map(Into::into),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ZoneOutlookHistoryResponse {
    pub zone_slug: String,
    pub season: String,
    /// Oldest first.
    pub issues: Vec<ZoneIssueResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct StoredZoneOutlookResponse {
    pub zone_slug: String,
    pub season: String,
    /// `YYYY-MM`.
    pub issued: String,
    pub outlook: Outlook,
    pub confidence_pct: f64,
    pub reason_en: Option<String>,
    pub reason_ku: Option<String>,
    pub updated_at: DateTime<Utc>,
}

/// The outlook as it is stored after an ingest.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct SavedZoneOutlookResponse {
    pub outlook: StoredZoneOutlookResponse,
}

impl From<&ZoneOutlook> for SavedZoneOutlookResponse {
    fn from(outlook: &ZoneOutlook) -> Self {
        Self {
            outlook: StoredZoneOutlookResponse {
                zone_slug: outlook.zone_slug().into(),
                season: outlook.season().into(),
                issued: outlook.issued().into(),
                outlook: (*outlook.outlook()).into(),
                confidence_pct: outlook.confidence().value(),
                reason_en: outlook.reason_en().as_ref().map(Into::into),
                reason_ku: outlook.reason_ku().as_ref().map(Into::into),
                updated_at: *outlook.updated_at(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct OutlookRunResponse {
    pub season: String,
    /// `YYYY-MM`.
    pub issued: String,
    pub seasons_tested: i32,
    pub seasons_right: i32,
    pub method: String,
    pub updated_at: DateTime<Utc>,
}

/// The track record as it is stored after an ingest.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct SavedOutlookRunResponse {
    pub run: OutlookRunResponse,
}

impl From<&OutlookRun> for SavedOutlookRunResponse {
    fn from(run: &OutlookRun) -> Self {
        Self {
            run: OutlookRunResponse {
                season: run.season().into(),
                issued: run.issued().into(),
                seasons_tested: *run.seasons_tested(),
                seasons_right: *run.seasons_right(),
                method: run.method().into(),
                updated_at: *run.updated_at(),
            },
        }
    }
}

/// A blank filter is the same as none: a filter form sends its empty fields.
fn filter(raw: Option<String>) -> Option<String> {
    raw.map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct OutlookDashboardQuery {
    /// Season like `2026-27`. Without it every season is listed.
    pub season: Option<String>,
    /// Issue month, `YYYY-MM`. Without it every issue is listed.
    pub issued: Option<String>,
}

impl OutlookDashboardQuery {
    pub fn into_input(self, pagination: Pagination) -> Result<ListZoneOutlooksInput, AppError> {
        Ok(ListZoneOutlooksInput {
            season: filter(self.season).map(Season::new).transpose()?,
            issued: filter(self.issued)
                .map(|issued| IssueMonth::new(&issued))
                .transpose()?,
            pagination,
        })
    }
}

/// The ingest body plus the key, which on a create is not in the path.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct CreateOutlookDashboardParams {
    /// Season like `2026-27`.
    pub season: String,
    /// Issue month, `YYYY-MM`.
    pub issued: String,
    pub zone_slug: String,
    pub outlook: Outlook,
    /// 0 to 100.
    pub confidence_pct: f64,
    /// One short sentence in English, 200 characters max.
    pub reason_en: Option<String>,
    /// The same sentence in Sorani, 200 characters max.
    pub reason_ku: Option<String>,
}

impl CreateOutlookDashboardParams {
    /// Goes through the ingest body, so that both are checked by one rule.
    pub fn into_input(self) -> Result<RecordZoneOutlookInput, AppError> {
        RecordZoneOutlookParams {
            outlook: self.outlook,
            confidence_pct: self.confidence_pct,
            reason_en: self.reason_en,
            reason_ku: self.reason_ku,
        }
        .into_input(self.season, self.issued, self.zone_slug)
    }
}

/// The ingest body plus the key, which on a create is not in the path.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct CreateOutlookRunDashboardParams {
    /// Season like `2026-27`.
    pub season: String,
    /// Issue month, `YYYY-MM`.
    pub issued: String,
    /// Past seasons the method was tested on.
    pub seasons_tested: i32,
    /// How many of them it called right. Never more than `seasons_tested`.
    pub seasons_right: i32,
    pub method: String,
}

impl CreateOutlookRunDashboardParams {
    /// Goes through the ingest body, so that both are checked by one rule.
    pub fn into_input(self) -> Result<RecordOutlookRunInput, AppError> {
        RecordOutlookRunParams {
            seasons_tested: self.seasons_tested,
            seasons_right: self.seasons_right,
            method: self.method,
        }
        .into_input(self.season, self.issued)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct OutlooksPageResponse {
    /// Newest issue first.
    pub outlooks: Vec<StoredZoneOutlookResponse>,
    /// How many outlooks match in all, on every page.
    pub count: u64,
    pub page: u64,
    pub rows_per_page: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct OutlookRunsResponse {
    /// Newest issue first.
    pub runs: Vec<OutlookRunResponse>,
}
