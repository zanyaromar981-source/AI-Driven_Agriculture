use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::{
    app::Pagination,
    features::dams::{
        app::{
            AppError,
            use_cases::{ListDamReadingsInput, RecordDamReadingInput, ViewDamHistoryInput},
        },
        domain::{Dam, DamReading, DamSlug, DamStatus, PercentFull, ReadingSource},
    },
    shared::DomainError,
};

/// Days travel as `YYYY-MM-DD`. They are parsed here rather than by the
/// extractor so that a bad one is answered like every other invalid value.
pub(super) fn parse_day(field: &str, raw: &str) -> Result<NaiveDate, AppError> {
    raw.parse().map_err(|_| {
        DomainError::InvalidValue(format!("{field} must be a day like 2026-10-08")).into()
    })
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DamHistoryQuery {
    /// First day, `YYYY-MM-DD`. Defaults to ten years before `to`.
    pub from: Option<String>,
    /// Last day, `YYYY-MM-DD`. Defaults to today.
    pub to: Option<String>,
}

impl DamHistoryQuery {
    pub fn into_input(self, slug: DamSlug) -> Result<ViewDamHistoryInput, AppError> {
        Ok(ViewDamHistoryInput {
            slug,
            from: self.from.map(|raw| parse_day("from", &raw)).transpose()?,
            to: self.to.map(|raw| parse_day("to", &raw)).transpose()?,
        })
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct RecordDamReadingParams {
    /// 0 to 100.
    pub pct_full: f64,
    /// Billion m3. At most 5% above the dam's capacity.
    pub volume_bn_m3: Option<f64>,
    pub lake_area_km2: Option<f64>,
    /// Billion m3 that can go to farms this season.
    pub farm_supply_bn_m3: Option<f64>,
    /// The job or data set the numbers come from.
    pub source: String,
}

impl RecordDamReadingParams {
    pub fn into_input(self, slug: String, day: String) -> Result<RecordDamReadingInput, AppError> {
        Ok(RecordDamReadingInput {
            slug: DamSlug::new(slug)?,
            day: parse_day("day", &day)?,
            pct_full: PercentFull::new(self.pct_full)?,
            volume_bn_m3: self.volume_bn_m3,
            lake_area_km2: self.lake_area_km2,
            farm_supply_bn_m3: self.farm_supply_bn_m3,
            source: ReadingSource::new(self.source)?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct LatestReadingResponse {
    pub day: NaiveDate,
    pub pct_full: f64,
    pub volume_bn_m3: Option<f64>,
    pub lake_area_km2: Option<f64>,
    pub farm_supply_bn_m3: Option<f64>,
    pub source: String,
}

impl From<&DamReading> for LatestReadingResponse {
    fn from(reading: &DamReading) -> Self {
        Self {
            day: *reading.day(),
            pct_full: reading.pct_full().value(),
            volume_bn_m3: *reading.volume_bn_m3(),
            lake_area_km2: *reading.lake_area_km2(),
            farm_supply_bn_m3: *reading.farm_supply_bn_m3(),
            source: reading.source().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct YearAgoReadingResponse {
    pub day: NaiveDate,
    pub pct_full: f64,
}

impl From<&DamReading> for YearAgoReadingResponse {
    fn from(reading: &DamReading) -> Self {
        Self {
            day: *reading.day(),
            pct_full: reading.pct_full().value(),
        }
    }
}

/// `latest` is null until a job has sent a reading. `year_ago` is null when
/// no reading lies within 45 days of one year before the latest.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DamResponse {
    pub slug: String,
    pub name_en: String,
    pub name_ku: String,
    pub capacity_bn_m3: f64,
    pub latest: Option<LatestReadingResponse>,
    pub year_ago: Option<YearAgoReadingResponse>,
}

impl From<&DamStatus> for DamResponse {
    fn from(status: &DamStatus) -> Self {
        let dam: &Dam = status.dam();

        Self {
            slug: dam.slug().into(),
            name_en: dam.name_en().clone(),
            name_ku: dam.name_ku().clone(),
            capacity_bn_m3: *dam.capacity_bn_m3(),
            latest: status.latest().as_ref().map(Into::into),
            year_ago: status.year_ago().as_ref().map(Into::into),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DamsResponse {
    pub dams: Vec<DamResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct HistoryReadingResponse {
    pub day: NaiveDate,
    pub pct_full: f64,
    pub volume_bn_m3: Option<f64>,
}

impl From<&DamReading> for HistoryReadingResponse {
    fn from(reading: &DamReading) -> Self {
        Self {
            day: *reading.day(),
            pct_full: reading.pct_full().value(),
            volume_bn_m3: *reading.volume_bn_m3(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DamHistoryResponse {
    pub slug: String,
    /// Oldest first.
    pub readings: Vec<HistoryReadingResponse>,
}

impl From<(&Dam, &[DamReading])> for DamHistoryResponse {
    fn from((dam, readings): (&Dam, &[DamReading])) -> Self {
        Self {
            slug: dam.slug().into(),
            readings: readings.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DamReadingResponse {
    pub day: NaiveDate,
    pub pct_full: f64,
    pub volume_bn_m3: Option<f64>,
    pub lake_area_km2: Option<f64>,
    pub farm_supply_bn_m3: Option<f64>,
    pub source: String,
    pub updated_at: DateTime<Utc>,
}

impl From<&DamReading> for DamReadingResponse {
    fn from(reading: &DamReading) -> Self {
        Self {
            day: *reading.day(),
            pct_full: reading.pct_full().value(),
            volume_bn_m3: *reading.volume_bn_m3(),
            lake_area_km2: *reading.lake_area_km2(),
            farm_supply_bn_m3: *reading.farm_supply_bn_m3(),
            source: reading.source().into(),
            updated_at: *reading.updated_at(),
        }
    }
}

/// The reading as it is stored after an ingest.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct SavedDamReadingResponse {
    pub slug: String,
    pub reading: DamReadingResponse,
}

/// A blank filter is the same as none: a filter form sends its empty fields.
fn filter_day(field: &str, raw: Option<String>) -> Result<Option<NaiveDate>, AppError> {
    raw.filter(|raw| !raw.trim().is_empty())
        .map(|raw| parse_day(field, raw.trim()))
        .transpose()
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DamDashboardReadingsQuery {
    /// First day, `YYYY-MM-DD`. Without it the list starts at the oldest reading.
    pub from: Option<String>,
    /// Last day, `YYYY-MM-DD`. Without it the list ends at the newest reading.
    pub to: Option<String>,
}

impl DamDashboardReadingsQuery {
    pub fn into_input(
        self,
        slug: DamSlug,
        pagination: Pagination,
    ) -> Result<ListDamReadingsInput, AppError> {
        Ok(ListDamReadingsInput {
            slug,
            from: filter_day("from", self.from)?,
            to: filter_day("to", self.to)?,
            pagination,
        })
    }
}

/// The ingest body plus the day, which on the dashboard is not in the path.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct CreateDamDashboardReadingParams {
    /// Day of the reading, `YYYY-MM-DD`.
    pub day: String,
    /// 0 to 100.
    pub pct_full: f64,
    /// Billion m3. At most 5% above the dam's capacity.
    pub volume_bn_m3: Option<f64>,
    pub lake_area_km2: Option<f64>,
    /// Billion m3 that can go to farms this season.
    pub farm_supply_bn_m3: Option<f64>,
    /// Who or what the numbers come from.
    pub source: String,
}

impl CreateDamDashboardReadingParams {
    /// Goes through the ingest body, so that both are checked by one rule.
    pub fn into_input(self, slug: String) -> Result<RecordDamReadingInput, AppError> {
        RecordDamReadingParams {
            pct_full: self.pct_full,
            volume_bn_m3: self.volume_bn_m3,
            lake_area_km2: self.lake_area_km2,
            farm_supply_bn_m3: self.farm_supply_bn_m3,
            source: self.source,
        }
        .into_input(slug, self.day)
    }
}

/// A dam as reference data, without readings.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DamReferenceResponse {
    pub slug: String,
    pub name_en: String,
    pub name_ku: String,
    pub capacity_bn_m3: f64,
}

impl From<&Dam> for DamReferenceResponse {
    fn from(dam: &Dam) -> Self {
        Self {
            slug: dam.slug().into(),
            name_en: dam.name_en().clone(),
            name_ku: dam.name_ku().clone(),
            capacity_bn_m3: *dam.capacity_bn_m3(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DamReferencesResponse {
    pub dams: Vec<DamReferenceResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DamReadingsPageResponse {
    /// Newest day first.
    pub readings: Vec<DamReadingResponse>,
    /// How many readings match in all, on every page.
    pub count: u64,
    pub page: u64,
    pub rows_per_page: u64,
}
