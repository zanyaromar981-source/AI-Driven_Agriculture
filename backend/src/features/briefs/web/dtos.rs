use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::{
    app::Pagination,
    features::briefs::{
        app::{
            AppError,
            use_cases::{ListBriefsInput, ListStoredBriefsInput, RecordBriefInput},
        },
        domain::{
            self, Author, BriefPoint, BriefScope, BriefSource, BriefSummary, DailyBrief, FarmBrief,
            FarmZone, Headline, PointText, SourceTitle, SourceUrl, ZoneSlug,
        },
    },
    shared::DomainError,
};

/// Days travel as `YYYY-MM-DD`. They are parsed here rather than by the
/// extractor so that a bad one is answered like every other invalid value.
pub(super) fn parse_day(field: &str, raw: &str) -> Result<NaiveDate, AppError> {
    raw.parse().map_err(|_| {
        DomainError::InvalidValue(format!("{field} must be a day like 2026-10-09")).into()
    })
}

/// A blank filter is the same as none: a filter form sends its empty fields.
fn filled(raw: Option<String>) -> Option<String> {
    raw.map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
}

fn filter_day(field: &str, raw: Option<String>) -> Result<Option<NaiveDate>, AppError> {
    filled(raw).map(|raw| parse_day(field, &raw)).transpose()
}

pub(super) fn scope(raw: String) -> Result<BriefScope, AppError> {
    Ok(BriefScope::new(raw)?)
}

/// The scope a public read is about. Left out, it is the whole region.
fn scope_or_region(raw: Option<String>) -> Result<BriefScope, AppError> {
    filled(raw).map_or_else(|| Ok(BriefScope::region()), scope)
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct BriefLatestQuery {
    /// `region` or a zone slug such as `chamchamal`. Defaults to `region`.
    pub scope: Option<String>,
}

impl BriefLatestQuery {
    pub fn into_scope(self) -> Result<BriefScope, AppError> {
        scope_or_region(self.scope)
    }
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct BriefListQuery {
    /// `region` or a zone slug such as `chamchamal`. Defaults to `region`.
    pub scope: Option<String>,
    /// First day, `YYYY-MM-DD`. Defaults to the start of the last 14 days.
    pub from: Option<String>,
    /// Last day, `YYYY-MM-DD`. Defaults to now. At most 92 days after `from`.
    pub to: Option<String>,
}

impl BriefListQuery {
    pub fn into_input(self) -> Result<ListBriefsInput, AppError> {
        Ok(ListBriefsInput {
            scope: scope_or_region(self.scope)?,
            from: filter_day("from", self.from)?,
            to: filter_day("to", self.to)?,
        })
    }
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct BriefDashboardListQuery {
    /// `region` or a zone slug. Without it the list holds every scope.
    pub scope: Option<String>,
    /// First day, `YYYY-MM-DD`. Without it the list starts at the oldest brief.
    pub from: Option<String>,
    /// Last day, `YYYY-MM-DD`. Without it the list ends at the newest brief.
    pub to: Option<String>,
}

impl BriefDashboardListQuery {
    pub fn into_input(self, pagination: Pagination) -> Result<ListStoredBriefsInput, AppError> {
        Ok(ListStoredBriefsInput {
            scope: filled(self.scope).map(scope).transpose()?,
            from: filter_day("from", self.from)?,
            to: filter_day("to", self.to)?,
            pagination,
        })
    }
}

/// How much attention one point of a brief asks for.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BriefPointLevel {
    Info,
    Watch,
    Alarm,
}

impl From<BriefPointLevel> for domain::PointLevel {
    fn from(value: BriefPointLevel) -> Self {
        match value {
            BriefPointLevel::Info => domain::PointLevel::Info,
            BriefPointLevel::Watch => domain::PointLevel::Watch,
            BriefPointLevel::Alarm => domain::PointLevel::Alarm,
        }
    }
}

impl From<domain::PointLevel> for BriefPointLevel {
    fn from(value: domain::PointLevel) -> Self {
        match value {
            domain::PointLevel::Info => BriefPointLevel::Info,
            domain::PointLevel::Watch => BriefPointLevel::Watch,
            domain::PointLevel::Alarm => BriefPointLevel::Alarm,
        }
    }
}

/// One thing the brief wants the reader to notice.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct BriefPointParams {
    pub level: BriefPointLevel,
    /// 1 to 300 characters.
    pub text_en: String,
    /// 1 to 300 characters.
    pub text_ku: String,
}

impl BriefPointParams {
    fn into_point(self) -> Result<BriefPoint, AppError> {
        Ok(BriefPoint::new(
            self.level.into(),
            PointText::new(self.text_en)?,
            PointText::new(self.text_ku)?,
        ))
    }
}

/// A page the author read while writing the brief.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct BriefSourceParams {
    /// 1 to 200 characters.
    pub title: String,
    /// Starts with `https://` or `http://`, at most 500 characters.
    pub url: String,
}

impl BriefSourceParams {
    fn into_source(self) -> Result<BriefSource, AppError> {
        Ok(BriefSource::new(
            SourceTitle::new(self.title)?,
            SourceUrl::new(self.url)?,
        ))
    }
}

/// The brief of one day for one scope. It replaces the brief already stored
/// for that day and scope as a whole.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct RecordBriefParams {
    /// 1 to 120 characters.
    pub headline_en: String,
    /// 1 to 120 characters.
    pub headline_ku: String,
    /// 1 to 2,000 characters.
    pub summary_en: String,
    /// 1 to 2,000 characters.
    pub summary_ku: String,
    /// 0 to 8 points.
    pub points: Vec<BriefPointParams>,
    /// 0 to 12 sources.
    pub sources: Vec<BriefSourceParams>,
    /// Which tool or model wrote it, 1 to 80 characters.
    pub author: String,
    /// When the author finished writing.
    pub generated_at: DateTime<Utc>,
}

impl RecordBriefParams {
    pub fn into_input(self, day: &str, raw_scope: String) -> Result<RecordBriefInput, AppError> {
        Ok(RecordBriefInput {
            day: parse_day("day", day)?,
            scope: scope(raw_scope)?,
            headline_en: Headline::new(self.headline_en)?,
            headline_ku: Headline::new(self.headline_ku)?,
            summary_en: BriefSummary::new(self.summary_en)?,
            summary_ku: BriefSummary::new(self.summary_ku)?,
            points: self
                .points
                .into_iter()
                .map(BriefPointParams::into_point)
                .collect::<Result<Vec<_>, _>>()?,
            sources: self
                .sources
                .into_iter()
                .map(BriefSourceParams::into_source)
                .collect::<Result<Vec<_>, _>>()?,
            author: Author::new(self.author)?,
            generated_at: self.generated_at,
        })
    }
}

/// Which district one farm lies in.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct BriefFarmZoneParams {
    /// The farm's id, as the coverage list gives it.
    pub farm_id: String,
    pub zone_slug: String,
}

impl BriefFarmZoneParams {
    fn into_farm_zone(self) -> Result<FarmZone, AppError> {
        // In a body, unlike in a path, an id that is not a number is a bad
        // value: there is no single thing the request could fail to find.
        let farm_id = self.farm_id.parse().map_err(|_| {
            DomainError::InvalidValue("farm_id must be a farm id like \"12\"".to_string())
        })?;

        Ok(FarmZone::new(farm_id, ZoneSlug::new(self.zone_slug)?)?)
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct RecordBriefFarmZonesParams {
    /// 1 to 2,000 farms, each at most once.
    pub farms: Vec<BriefFarmZoneParams>,
}

impl RecordBriefFarmZonesParams {
    pub fn into_farm_zones(self) -> Result<Vec<FarmZone>, AppError> {
        // The count is checked before any farm is parsed, so an over-long
        // push is refused without doing the work.
        FarmZone::check_count(self.farms.len())?;

        self.farms
            .into_iter()
            .map(BriefFarmZoneParams::into_farm_zone)
            .collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct BriefPointResponse {
    pub level: BriefPointLevel,
    pub text_en: String,
    pub text_ku: String,
}

impl From<&BriefPoint> for BriefPointResponse {
    fn from(point: &BriefPoint) -> Self {
        Self {
            level: (*point.level()).into(),
            text_en: point.text_en().into(),
            text_ku: point.text_ku().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct BriefSourceResponse {
    pub title: String,
    pub url: String,
}

impl From<&BriefSource> for BriefSourceResponse {
    fn from(source: &BriefSource) -> Self {
        Self {
            title: source.title().into(),
            url: source.url().into(),
        }
    }
}

/// A stored brief, exactly as its author wrote it.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct BriefResponse {
    pub day: NaiveDate,
    /// `region` or a zone slug.
    pub scope: String,
    pub headline_en: String,
    pub headline_ku: String,
    pub summary_en: String,
    pub summary_ku: String,
    pub points: Vec<BriefPointResponse>,
    pub sources: Vec<BriefSourceResponse>,
    /// Which tool or model wrote it.
    pub author: String,
    pub generated_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&DailyBrief> for BriefResponse {
    fn from(brief: &DailyBrief) -> Self {
        Self {
            day: *brief.day(),
            scope: brief.scope().into(),
            headline_en: brief.headline_en().into(),
            headline_ku: brief.headline_ku().into(),
            summary_en: brief.summary_en().into(),
            summary_ku: brief.summary_ku().into(),
            points: brief.points().iter().map(Into::into).collect(),
            sources: brief.sources().iter().map(Into::into).collect(),
            author: brief.author().into(),
            generated_at: *brief.generated_at(),
            updated_at: *brief.updated_at(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct OneBriefResponse {
    pub brief: BriefResponse,
}

impl From<&DailyBrief> for OneBriefResponse {
    fn from(brief: &DailyBrief) -> Self {
        Self {
            brief: brief.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct BriefsResponse {
    /// Newest day first.
    pub briefs: Vec<BriefResponse>,
}

/// The farm id is an opaque string to the app. `zone_slug` is the district
/// the farm is recorded in, or null when none is recorded yet. `brief` is
/// that district's newest brief, or the region's when the district has
/// none; its `scope` says which. Null means no brief is stored yet.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FarmBriefResponse {
    pub farm_id: String,
    pub zone_slug: Option<String>,
    pub brief: Option<BriefResponse>,
}

impl From<&FarmBrief> for FarmBriefResponse {
    fn from(farm_brief: &FarmBrief) -> Self {
        Self {
            farm_id: farm_brief.farm_id().to_string(),
            zone_slug: farm_brief.zone_slug().as_ref().map(Into::into),
            brief: farm_brief.brief().as_ref().map(Into::into),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct BriefFarmZonesRecordedResponse {
    pub recorded: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct BriefsPageResponse {
    /// Newest day first, then by scope.
    pub briefs: Vec<BriefResponse>,
    /// How many briefs match in all, on every page.
    pub count: u64,
    pub page: u64,
    pub rows_per_page: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::briefs::domain::{BriefError, PointLevel};

    fn params() -> RecordBriefParams {
        serde_json::from_value(serde_json::json!({
            "headline_en": "A dry week",
            "headline_ku": "هەفتەیەکی وشک",
            "summary_en": "No rain fell.",
            "summary_ku": "باران نەباری.",
            "points": [{"level": "alarm", "text_en": "Two fires", "text_ku": "دوو ئاگر"}],
            "sources": [{"title": "FAO", "url": "https://example.org"}],
            "author": "codex-cli gpt-5",
            "generated_at": "2026-10-08T21:04:00Z"
        }))
        .expect("params")
    }

    #[test]
    fn every_level_survives_the_trip_through_its_dto_spelled_as_it_is_stored() {
        for level in PointLevel::ALL {
            let dto = BriefPointLevel::from(level);

            assert_eq!(PointLevel::from(dto), level);
            assert_eq!(
                serde_json::to_value(dto).expect("json"),
                serde_json::Value::String(String::from(level))
            );
        }
    }

    #[test]
    fn a_level_the_backend_does_not_know_is_not_read_as_another() {
        assert!(serde_json::from_str::<BriefPointLevel>("\"danger\"").is_err());
    }

    #[test]
    fn the_day_and_scope_of_a_push_come_from_the_path() {
        let input = params()
            .into_input("2026-10-09", "chamchamal".to_string())
            .expect("input");

        assert_eq!(
            input.day,
            NaiveDate::from_ymd_opt(2026, 10, 9).expect("date")
        );
        assert_eq!(input.scope.as_str(), "chamchamal");
        assert_eq!(*input.points[0].level(), PointLevel::Alarm);
    }

    #[test]
    fn a_path_day_that_is_not_a_day_or_a_scope_that_is_not_a_slug_is_invalid() {
        assert!(
            params()
                .into_input("9 October", "region".to_string())
                .is_err()
        );
        assert!(
            params()
                .into_input("2026-02-30", "region".to_string())
                .is_err()
        );
        assert!(
            params()
                .into_input("2026-10-09", "All Zones".to_string())
                .is_err()
        );
    }

    #[test]
    fn a_public_read_without_a_scope_is_about_the_region() {
        for raw in [None, Some(String::new()), Some("  ".to_string())] {
            assert!(
                BriefLatestQuery { scope: raw }
                    .into_scope()
                    .expect("scope")
                    .is_region()
            );
        }

        assert_eq!(
            BriefLatestQuery {
                scope: Some("kalar".to_string())
            }
            .into_scope()
            .expect("scope")
            .as_str(),
            "kalar"
        );
    }

    #[test]
    fn a_dashboard_list_without_a_scope_is_about_every_scope() {
        let input = BriefDashboardListQuery {
            scope: Some(String::new()),
            from: Some(String::new()),
            to: None,
        }
        .into_input(Pagination::new(1, 20))
        .expect("input");

        assert!(input.scope.is_none());
        assert!(input.from.is_none());
    }

    #[test]
    fn a_farm_id_in_a_body_that_is_not_a_number_is_invalid() {
        let farms = |farm_id: &str| RecordBriefFarmZonesParams {
            farms: vec![BriefFarmZoneParams {
                farm_id: farm_id.to_string(),
                zone_slug: "kalar".to_string(),
            }],
        };

        assert!(farms("12").into_farm_zones().is_ok());
        assert!(farms("twelve").into_farm_zones().is_err());
        assert!(farms("0").into_farm_zones().is_err());
    }

    #[test]
    fn too_many_farms_are_refused_before_any_is_parsed() {
        let params = RecordBriefFarmZonesParams {
            farms: vec![
                BriefFarmZoneParams {
                    farm_id: "not a number".to_string(),
                    zone_slug: "Not A Slug".to_string(),
                };
                2_001
            ],
        };

        assert!(
            matches!(
                params.into_farm_zones(),
                Err(AppError::Brief(BriefError::FarmCount { .. }))
            ),
            "the count is the error, not the first unreadable farm"
        );
    }

    #[test]
    fn a_farm_brief_with_nothing_stored_says_so_with_nulls() {
        let response = FarmBriefResponse::from(&FarmBrief::new(7, None, None));
        let json = serde_json::to_value(&response).expect("json");

        assert_eq!(
            json,
            serde_json::json!({"farm_id": "7", "zone_slug": null, "brief": null})
        );
    }
}
