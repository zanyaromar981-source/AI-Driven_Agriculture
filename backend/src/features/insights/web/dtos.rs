use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::features::insights::{
    app::{AppError, use_cases::RecordFarmInsightInput},
    domain::{self, FarmCoverage, FarmInsight, InsightSource, Measure, MeasureCode, Summary},
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Topic {
    SurfaceWater,
    Groundwater,
    Soil,
    Rain,
    Dryness,
    Greenness,
    Weather,
}

impl Topic {
    /// The topic named by a path segment.
    pub fn from_path(raw: &str) -> Result<domain::Topic, AppError> {
        Ok(domain::Topic::try_from(raw)?)
    }
}

impl From<Topic> for domain::Topic {
    fn from(value: Topic) -> Self {
        match value {
            Topic::SurfaceWater => domain::Topic::SurfaceWater,
            Topic::Groundwater => domain::Topic::Groundwater,
            Topic::Soil => domain::Topic::Soil,
            Topic::Rain => domain::Topic::Rain,
            Topic::Dryness => domain::Topic::Dryness,
            Topic::Greenness => domain::Topic::Greenness,
            Topic::Weather => domain::Topic::Weather,
        }
    }
}

impl From<domain::Topic> for Topic {
    fn from(value: domain::Topic) -> Self {
        match value {
            domain::Topic::SurfaceWater => Topic::SurfaceWater,
            domain::Topic::Groundwater => Topic::Groundwater,
            domain::Topic::Soil => Topic::Soil,
            domain::Topic::Rain => Topic::Rain,
            domain::Topic::Dryness => Topic::Dryness,
            domain::Topic::Greenness => Topic::Greenness,
            domain::Topic::Weather => Topic::Weather,
        }
    }
}

/// How far the data job trusts its own reading.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Sure,
    Likely,
    Unsure,
}

impl From<Confidence> for domain::Confidence {
    fn from(value: Confidence) -> Self {
        match value {
            Confidence::Sure => domain::Confidence::Sure,
            Confidence::Likely => domain::Confidence::Likely,
            Confidence::Unsure => domain::Confidence::Unsure,
        }
    }
}

impl From<domain::Confidence> for Confidence {
    fn from(value: domain::Confidence) -> Self {
        match value {
            domain::Confidence::Sure => Confidence::Sure,
            domain::Confidence::Likely => Confidence::Likely,
            domain::Confidence::Unsure => Confidence::Unsure,
        }
    }
}

/// One named number of a reading.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct MeasureParams {
    /// Lower-case letters, digits and underscores, 1 to 40 characters,
    /// unique within the reading.
    pub code: String,
    pub value: f64,
    /// Up to 20 characters. Leave it out for a number without a unit.
    #[serde(default)]
    pub unit: String,
    pub label_en: String,
    pub label_ku: Option<String>,
}

impl MeasureParams {
    fn into_measure(self) -> Result<Measure, AppError> {
        Ok(Measure::new(
            MeasureCode::new(self.code)?,
            self.value,
            self.unit,
            self.label_en,
            self.label_ku,
        )?)
    }
}

/// A farm's reading for one topic. It replaces the reading the farm already
/// has for that topic as a whole.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct RecordFarmInsightParams {
    /// The day the numbers describe.
    pub as_of: NaiveDate,
    /// Where the numbers come from, 1 to 120 characters.
    pub source: String,
    pub confidence: Confidence,
    pub summary_en: Option<String>,
    pub summary_ku: Option<String>,
    /// 1 to 20 measures.
    pub measures: Vec<MeasureParams>,
}

impl RecordFarmInsightParams {
    pub fn into_input(
        self,
        farm_id: i32,
        topic: domain::Topic,
    ) -> Result<RecordFarmInsightInput, AppError> {
        Ok(RecordFarmInsightInput {
            farm_id,
            topic,
            as_of: self.as_of,
            source: InsightSource::new(self.source)?,
            confidence: self.confidence.into(),
            summary_en: self.summary_en.map(Summary::new).transpose()?,
            summary_ku: self.summary_ku.map(Summary::new).transpose()?,
            measures: self
                .measures
                .into_iter()
                .map(MeasureParams::into_measure)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct MeasureResponse {
    pub code: String,
    pub value: f64,
    pub unit: String,
    pub label_en: String,
    pub label_ku: Option<String>,
}

impl From<&Measure> for MeasureResponse {
    fn from(measure: &Measure) -> Self {
        Self {
            code: measure.code().into(),
            value: *measure.value(),
            unit: measure.unit().clone(),
            label_en: measure.label_en().clone(),
            label_ku: measure.label_ku().clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct TopicInsightResponse {
    pub topic: Topic,
    pub as_of: NaiveDate,
    pub source: String,
    pub confidence: Confidence,
    pub summary_en: Option<String>,
    pub summary_ku: Option<String>,
    pub measures: Vec<MeasureResponse>,
}

impl From<&FarmInsight> for TopicInsightResponse {
    fn from(insight: &FarmInsight) -> Self {
        Self {
            topic: (*insight.topic()).into(),
            as_of: *insight.as_of(),
            source: insight.source().into(),
            confidence: (*insight.confidence()).into(),
            summary_en: insight.summary_en().as_ref().map(Into::into),
            summary_ku: insight.summary_ku().as_ref().map(Into::into),
            measures: insight.measures().iter().map(Into::into).collect(),
        }
    }
}

/// The farm id is an opaque string to the app. `topics` holds only the
/// topics that have a reading; an empty list means there is no data yet.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FarmInsightsResponse {
    pub farm_id: String,
    pub topics: Vec<TopicInsightResponse>,
}

impl From<(i32, &[FarmInsight])> for FarmInsightsResponse {
    fn from((farm_id, insights): (i32, &[FarmInsight])) -> Self {
        Self {
            farm_id: farm_id.to_string(),
            topics: insights.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct TopicStampResponse {
    pub topic: Topic,
    pub as_of: NaiveDate,
}

/// One farm as a data job sees it: where it is, how large, and which
/// readings it already has. `lat` and `lon` are the farm's centroid. The
/// owner is deliberately not part of it.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FarmCoverageResponse {
    pub id: String,
    pub lat: f64,
    pub lon: f64,
    pub area_dunam: f64,
    pub topics: Vec<TopicStampResponse>,
}

impl From<&FarmCoverage> for FarmCoverageResponse {
    fn from(coverage: &FarmCoverage) -> Self {
        Self {
            id: coverage.site().farm_id().to_string(),
            lat: *coverage.site().lat(),
            lon: *coverage.site().lon(),
            area_dunam: *coverage.site().area_dunam(),
            topics: coverage
                .readings()
                .iter()
                .map(|(topic, as_of)| TopicStampResponse {
                    topic: (*topic).into(),
                    as_of: *as_of,
                })
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FarmsCoverageResponse {
    pub farms: Vec<FarmCoverageResponse>,
}

/// A reading a staff member enters by hand: the ingest body plus the topic
/// it is about.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct InsightDashboardCreateParams {
    pub topic: Topic,
    #[serde(flatten)]
    pub reading: RecordFarmInsightParams,
}

impl InsightDashboardCreateParams {
    pub fn into_input(self, farm_id: i32) -> Result<RecordFarmInsightInput, AppError> {
        self.reading.into_input(farm_id, self.topic.into())
    }
}

/// A stored reading as the dashboard's editing screen sees it. The owner of
/// the farm is deliberately not part of it.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct InsightDashboardResponse {
    pub topic: Topic,
    pub as_of: NaiveDate,
    pub source: String,
    pub confidence: Confidence,
    pub summary_en: Option<String>,
    pub summary_ku: Option<String>,
    pub measures: Vec<MeasureResponse>,
    pub updated_at: DateTime<Utc>,
}

impl From<&FarmInsight> for InsightDashboardResponse {
    fn from(insight: &FarmInsight) -> Self {
        Self {
            topic: (*insight.topic()).into(),
            as_of: *insight.as_of(),
            source: insight.source().into(),
            confidence: (*insight.confidence()).into(),
            summary_en: insight.summary_en().as_ref().map(Into::into),
            summary_ku: insight.summary_ku().as_ref().map(Into::into),
            measures: insight.measures().iter().map(Into::into).collect(),
            updated_at: *insight.updated_at(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct InsightDashboardOneResponse {
    pub farm_id: String,
    pub insight: InsightDashboardResponse,
}

/// `insights` holds only the topics that have a reading, in the fixed topic
/// order.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct InsightDashboardListResponse {
    pub farm_id: String,
    pub insights: Vec<InsightDashboardResponse>,
}
