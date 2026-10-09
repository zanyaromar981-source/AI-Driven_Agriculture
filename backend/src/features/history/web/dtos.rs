use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::features::history::{
    app::{
        AppError,
        use_cases::{FarmHistoryQuery, RecordFarmHistoryInput},
    },
    domain::{self, FarmCoverage, HistorySource, Month, Series, SeriesUpload},
};

/// One thing kept month by month for a farm. Each has one fixed unit:
/// `rain_mm` and `et0_mm` in mm, the temperatures in °C, `soil_moisture` in
/// m3/m3, `greenness` as NDVI (`ndvi`), `groundwater_pct` as a percentile.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum HistoryMetric {
    RainMm,
    TempMaxC,
    TempMinC,
    Et0Mm,
    SoilMoisture,
    Greenness,
    GroundwaterPct,
}

impl HistoryMetric {
    /// The metric named by a path segment.
    pub fn from_path(raw: &str) -> Result<domain::Metric, AppError> {
        Ok(domain::Metric::try_from(raw)?)
    }
}

impl From<HistoryMetric> for domain::Metric {
    fn from(value: HistoryMetric) -> Self {
        match value {
            HistoryMetric::RainMm => domain::Metric::RainMm,
            HistoryMetric::TempMaxC => domain::Metric::TempMaxC,
            HistoryMetric::TempMinC => domain::Metric::TempMinC,
            HistoryMetric::Et0Mm => domain::Metric::Et0Mm,
            HistoryMetric::SoilMoisture => domain::Metric::SoilMoisture,
            HistoryMetric::Greenness => domain::Metric::Greenness,
            HistoryMetric::GroundwaterPct => domain::Metric::GroundwaterPct,
        }
    }
}

impl From<domain::Metric> for HistoryMetric {
    fn from(value: domain::Metric) -> Self {
        match value {
            domain::Metric::RainMm => HistoryMetric::RainMm,
            domain::Metric::TempMaxC => HistoryMetric::TempMaxC,
            domain::Metric::TempMinC => HistoryMetric::TempMinC,
            domain::Metric::Et0Mm => HistoryMetric::Et0Mm,
            domain::Metric::SoilMoisture => HistoryMetric::SoilMoisture,
            domain::Metric::Greenness => HistoryMetric::Greenness,
            domain::Metric::GroundwaterPct => HistoryMetric::GroundwaterPct,
        }
    }
}

/// Which part of a farm's history to return. Everything is optional.
#[derive(Deserialize, Debug, Clone, Default, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct HistoryQueryParams {
    /// Metric names separated by commas, for example `rain_mm,greenness`.
    /// Left out: every metric.
    pub metrics: Option<String>,
    /// First month, `YYYY-MM`. Left out: 120 months before `to`, counting
    /// `to`.
    pub from: Option<String>,
    /// Last month, `YYYY-MM`. Left out: the last full month. A window holds
    /// 120 months at most.
    pub to: Option<String>,
}

impl HistoryQueryParams {
    pub fn into_query(self) -> Result<FarmHistoryQuery, AppError> {
        Ok(FarmHistoryQuery {
            metrics: self
                .metrics
                .as_deref()
                .map(domain::Metric::parse_list)
                .transpose()?,
            from: self.from.as_deref().map(Month::parse).transpose()?,
            to: self.to.as_deref().map(Month::parse).transpose()?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct HistoryPointParams {
    /// `YYYY-MM`.
    pub month: String,
    pub value: f64,
}

/// Months of one metric of one farm. A month already stored is replaced; a
/// stored month that is not in `points` is kept.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct RecordFarmHistoryParams {
    /// Must be the metric's own unit.
    pub unit: String,
    /// What was measured, by whom and at what resolution, 1 to 200
    /// characters.
    pub source: String,
    /// When the job read the numbers from its source.
    pub as_of: DateTime<Utc>,
    /// 1 to 240 months, none of them twice.
    pub points: Vec<HistoryPointParams>,
}

impl RecordFarmHistoryParams {
    pub fn into_input(
        self,
        farm_id: i32,
        metric: domain::Metric,
    ) -> Result<RecordFarmHistoryInput, AppError> {
        Ok(RecordFarmHistoryInput {
            farm_id,
            metric,
            unit: self.unit,
            source: HistorySource::new(self.source)?,
            as_of: self.as_of,
            points: self
                .points
                .into_iter()
                .map(|point| Ok((Month::parse(&point.month)?, point.value)))
                .collect::<Result<Vec<_>, AppError>>()?,
        })
    }
}

/// What a push stored.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct HistoryRecordedResponse {
    pub farm_id: String,
    pub metric: HistoryMetric,
    /// How many months the push carried.
    pub points: usize,
    pub first_month: String,
    pub last_month: String,
}

impl From<&SeriesUpload> for HistoryRecordedResponse {
    fn from(upload: &SeriesUpload) -> Self {
        let month = |point: Option<&domain::MonthlyPoint>| {
            point
                .map(|point| point.month().to_string())
                .unwrap_or_default()
        };

        Self {
            farm_id: upload.farm_id().to_string(),
            metric: (*upload.metric()).into(),
            points: upload.points().len(),
            first_month: month(upload.points().first()),
            last_month: month(upload.points().last()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct HistoryMonthResponse {
    /// `YYYY-MM`.
    pub month: String,
    pub value: f64,
}

/// The figure of one calendar year: the sum of its months for `rain_mm` and
/// `et0_mm`, given only when all twelve are stored; the mean of the months
/// it has for the other metrics. `months` says how many months it is made
/// from.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct HistoryYearResponse {
    pub year: i32,
    pub value: f64,
    pub months: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct HistorySeriesResponse {
    pub metric: HistoryMetric,
    pub unit: String,
    /// What was measured, by whom and at what resolution.
    pub source: String,
    /// When the data job last read the numbers from its source.
    pub as_of: DateTime<Utc>,
    /// Earliest first. A month nobody measured is not in the list.
    pub months: Vec<HistoryMonthResponse>,
    pub years: Vec<HistoryYearResponse>,
    /// Twelve entries, January first: the mean of each calendar month over
    /// the months returned, or null for a calendar month with none.
    pub normal: Vec<Option<f64>>,
}

impl From<&Series> for HistorySeriesResponse {
    fn from(series: &Series) -> Self {
        Self {
            metric: (*series.metric()).into(),
            unit: series.metric().unit().to_string(),
            source: series.source().into(),
            as_of: *series.as_of(),
            months: series
                .points()
                .iter()
                .map(|point| HistoryMonthResponse {
                    month: point.month().to_string(),
                    value: *point.value(),
                })
                .collect(),
            years: series
                .years()
                .iter()
                .map(|year| HistoryYearResponse {
                    year: *year.year(),
                    value: *year.value(),
                    months: *year.months(),
                })
                .collect(),
            normal: series.normal().to_vec(),
        }
    }
}

/// The farm id is an opaque string to the app. `series` holds only the
/// metrics that have stored months in the window; an empty list means the
/// history is still being collected.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FarmHistoryResponse {
    pub farm_id: String,
    pub series: Vec<HistorySeriesResponse>,
}

impl From<(i32, &[Series])> for FarmHistoryResponse {
    fn from((farm_id, series): (i32, &[Series])) -> Self {
        Self {
            farm_id: farm_id.to_string(),
            series: series.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct HistoryMetricCoverageResponse {
    /// `YYYY-MM`.
    pub first_month: String,
    /// `YYYY-MM`.
    pub last_month: String,
    /// How many months are stored. Fewer than the span from first to last
    /// means there are holes.
    pub months: u32,
    pub as_of: DateTime<Utc>,
}

/// One farm as the history job sees it: where it is, since when, and what
/// is stored of each metric. `lat` and `lon` are the farm's centroid. A
/// metric with nothing stored is not a key of `metrics`. The owner is
/// deliberately not part of it.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct HistoryFarmCoverageResponse {
    pub farm_id: String,
    pub lat: f64,
    pub lon: f64,
    pub created_at: DateTime<Utc>,
    pub metrics: BTreeMap<String, HistoryMetricCoverageResponse>,
}

impl From<&FarmCoverage> for HistoryFarmCoverageResponse {
    fn from(coverage: &FarmCoverage) -> Self {
        Self {
            farm_id: coverage.site().farm_id().to_string(),
            lat: *coverage.site().lat(),
            lon: *coverage.site().lon(),
            created_at: *coverage.site().created_at(),
            metrics: coverage
                .metrics()
                .iter()
                .map(|metric| {
                    (
                        String::from(*metric.metric()),
                        HistoryMetricCoverageResponse {
                            first_month: metric.first_month().to_string(),
                            last_month: metric.last_month().to_string(),
                            months: *metric.months(),
                            as_of: *metric.as_of(),
                        },
                    )
                })
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct HistoryCoverageResponse {
    pub farms: Vec<HistoryFarmCoverageResponse>,
}
