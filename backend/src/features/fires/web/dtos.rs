use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::fires::{
        app::{
            AppError,
            use_cases::{CorrectFireInput, ListStoredFiresInput, RecordFireInput},
        },
        domain::{
            self, ExternalId, Fire, FireLocation, FireSource, FireSummary, PlaceName, WindowHours,
            ZoneSlug,
        },
    },
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FireStatus {
    Active,
    Spreading,
    UnderControl,
    Out,
}

impl From<FireStatus> for domain::FireStatus {
    fn from(value: FireStatus) -> Self {
        match value {
            FireStatus::Active => domain::FireStatus::Active,
            FireStatus::Spreading => domain::FireStatus::Spreading,
            FireStatus::UnderControl => domain::FireStatus::UnderControl,
            FireStatus::Out => domain::FireStatus::Out,
        }
    }
}

impl From<domain::FireStatus> for FireStatus {
    fn from(value: domain::FireStatus) -> Self {
        match value {
            domain::FireStatus::Active => FireStatus::Active,
            domain::FireStatus::Spreading => FireStatus::Spreading,
            domain::FireStatus::UnderControl => FireStatus::UnderControl,
            domain::FireStatus::Out => FireStatus::Out,
        }
    }
}

/// The compass point the wind blows towards.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WindDirection {
    N,
    Ne,
    E,
    Se,
    S,
    Sw,
    W,
    Nw,
}

impl From<WindDirection> for domain::WindDirection {
    fn from(value: WindDirection) -> Self {
        match value {
            WindDirection::N => domain::WindDirection::N,
            WindDirection::Ne => domain::WindDirection::Ne,
            WindDirection::E => domain::WindDirection::E,
            WindDirection::Se => domain::WindDirection::Se,
            WindDirection::S => domain::WindDirection::S,
            WindDirection::Sw => domain::WindDirection::Sw,
            WindDirection::W => domain::WindDirection::W,
            WindDirection::Nw => domain::WindDirection::Nw,
        }
    }
}

impl From<domain::WindDirection> for WindDirection {
    fn from(value: domain::WindDirection) -> Self {
        match value {
            domain::WindDirection::N => WindDirection::N,
            domain::WindDirection::Ne => WindDirection::Ne,
            domain::WindDirection::E => WindDirection::E,
            domain::WindDirection::Se => WindDirection::Se,
            domain::WindDirection::S => WindDirection::S,
            domain::WindDirection::Sw => WindDirection::Sw,
            domain::WindDirection::W => WindDirection::W,
            domain::WindDirection::Nw => WindDirection::Nw,
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct FiresQueryDto {
    /// How many hours back to look, 1 to 168. 24 when left out.
    #[param(example = 24)]
    pub hours: Option<i64>,
}

impl FiresQueryDto {
    pub fn into_window(self) -> Result<WindowHours, AppError> {
        match self.hours {
            Some(hours) => Ok(WindowHours::new(hours)?),
            None => Ok(WindowHours::default()),
        }
    }
}

/// Everything the job knows about one fire. A field that is left out or
/// `null` is stored as unknown, also when an earlier push had a value.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct RecordFireParams {
    /// WGS84 decimal degrees, 28 to 40.
    pub lat: f64,
    /// WGS84 decimal degrees, 38 to 50.
    pub lon: f64,
    pub zone_slug: Option<String>,
    pub place_en: Option<String>,
    pub place_ku: Option<String>,
    pub detected_at: DateTime<Utc>,
    pub area_ha: Option<f64>,
    pub wind_kmh: Option<f64>,
    pub wind_direction: Option<WindDirection>,
    pub status: FireStatus,
    pub farms_within_5km: Option<i32>,
    pub farmers_alerted: Option<i32>,
    pub source: String,
}

impl RecordFireParams {
    pub fn into_input(self, external_id: String) -> Result<RecordFireInput, AppError> {
        Ok(RecordFireInput {
            external_id: ExternalId::new(external_id)?,
            location: FireLocation::new(self.lat, self.lon)?,
            zone_slug: self.zone_slug.map(ZoneSlug::new).transpose()?,
            place_en: self.place_en.map(PlaceName::new).transpose()?,
            place_ku: self.place_ku.map(PlaceName::new).transpose()?,
            detected_at: self.detected_at,
            area_ha: self.area_ha,
            wind_kmh: self.wind_kmh,
            wind_direction: self.wind_direction.map(Into::into),
            status: self.status.into(),
            farms_within_5km: self.farms_within_5km,
            farmers_alerted: self.farmers_alerted,
            source: FireSource::new(self.source)?,
        })
    }

    /// The same body as a staff member's correction of a stored fire, whose
    /// external id cannot change.
    pub fn into_correction_input(self) -> Result<CorrectFireInput, AppError> {
        Ok(CorrectFireInput {
            location: FireLocation::new(self.lat, self.lon)?,
            zone_slug: self.zone_slug.map(ZoneSlug::new).transpose()?,
            place_en: self.place_en.map(PlaceName::new).transpose()?,
            place_ku: self.place_ku.map(PlaceName::new).transpose()?,
            detected_at: self.detected_at,
            area_ha: self.area_ha,
            wind_kmh: self.wind_kmh,
            wind_direction: self.wind_direction.map(Into::into),
            status: self.status.into(),
            farms_within_5km: self.farms_within_5km,
            farmers_alerted: self.farmers_alerted,
            source: FireSource::new(self.source)?,
        })
    }
}

/// The id is an opaque string to the dashboard.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FireResponse {
    pub id: String,
    pub lat: f64,
    pub lon: f64,
    pub zone_slug: Option<String>,
    pub place_en: Option<String>,
    pub place_ku: Option<String>,
    pub detected_at: DateTime<Utc>,
    pub area_ha: Option<f64>,
    pub wind_kmh: Option<f64>,
    pub wind_direction: Option<WindDirection>,
    pub status: FireStatus,
    pub farms_within_5km: Option<i32>,
    pub farmers_alerted: Option<i32>,
    pub source: String,
}

impl TryFrom<&Fire> for FireResponse {
    type Error = AppError;

    fn try_from(fire: &Fire) -> Result<Self, Self::Error> {
        let id = fire.id().ok_or_else(|| {
            AppError::GlobalAppError(GlobalAppError::MissingValue(
                "Fire is missing its id".to_string(),
            ))
        })?;

        Ok(Self {
            id: id.to_string(),
            lat: fire.location().lat(),
            lon: fire.location().lon(),
            zone_slug: fire.zone_slug().as_ref().map(Into::into),
            place_en: fire.place_en().as_ref().map(Into::into),
            place_ku: fire.place_ku().as_ref().map(Into::into),
            detected_at: *fire.detected_at(),
            area_ha: *fire.area_ha(),
            wind_kmh: *fire.wind_kmh(),
            wind_direction: fire.wind_direction().map(Into::into),
            status: (*fire.status()).into(),
            farms_within_5km: *fire.farms_within_5km(),
            farmers_alerted: *fire.farmers_alerted(),
            source: fire.source().into(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FireSummaryResponse {
    /// Fires that are active or spreading.
    pub active: usize,
    pub under_control: usize,
    pub area_ha: f64,
    pub farms_within_5km: i64,
    pub farmers_alerted: i64,
    /// Zones with an active or spreading fire.
    pub zones: Vec<String>,
}

impl From<&FireSummary> for FireSummaryResponse {
    fn from(summary: &FireSummary) -> Self {
        Self {
            active: *summary.active(),
            under_control: *summary.under_control(),
            area_ha: *summary.area_ha(),
            farms_within_5km: *summary.farms_within_5km(),
            farmers_alerted: *summary.farmers_alerted(),
            zones: summary.zones().iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FiresResponse {
    pub hours: i64,
    pub fires: Vec<FireResponse>,
    pub summary: FireSummaryResponse,
}

impl TryFrom<(WindowHours, &[Fire], &FireSummary)> for FiresResponse {
    type Error = AppError;

    fn try_from(
        (window, fires, summary): (WindowHours, &[Fire], &FireSummary),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            hours: window.hours(),
            fires: fires
                .iter()
                .map(FireResponse::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            summary: summary.into(),
        })
    }
}

/// Which stored fires the dashboard's editing screen asks for.
#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct FireDashboardQuery {
    /// Earliest detection time, inclusive. 30 days before `to` when left out.
    pub from: Option<DateTime<Utc>>,
    /// Latest detection time, inclusive. Now when left out.
    pub to: Option<DateTime<Utc>>,
    /// `active`, `spreading`, `under_control` or `out`. Every status when
    /// left out.
    pub status: Option<String>,
    /// Zone slug, for example `chamchamal`.
    pub zone_slug: Option<String>,
}

impl FireDashboardQuery {
    pub fn into_input(self, pagination: Pagination) -> Result<ListStoredFiresInput, AppError> {
        Ok(ListStoredFiresInput {
            from: self.from,
            to: self.to,
            status: self
                .status
                .as_deref()
                .map(domain::FireStatus::try_from)
                .transpose()?,
            zone_slug: self.zone_slug.map(ZoneSlug::new).transpose()?,
            pagination,
        })
    }
}

/// A fire a staff member enters by hand: the ingest body plus the key it is
/// stored under.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct FireDashboardCreateParams {
    /// 1 to 100 printable ASCII characters, unique among the stored fires.
    pub external_id: String,
    #[serde(flatten)]
    pub fire: RecordFireParams,
}

impl FireDashboardCreateParams {
    pub fn into_input(self) -> Result<RecordFireInput, AppError> {
        self.fire.into_input(self.external_id)
    }
}

/// A stored fire with every field, for the dashboard's editing screen.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FireDashboardResponse {
    pub id: String,
    pub external_id: String,
    pub lat: f64,
    pub lon: f64,
    pub zone_slug: Option<String>,
    pub place_en: Option<String>,
    pub place_ku: Option<String>,
    pub detected_at: DateTime<Utc>,
    pub area_ha: Option<f64>,
    pub wind_kmh: Option<f64>,
    pub wind_direction: Option<WindDirection>,
    pub status: FireStatus,
    pub farms_within_5km: Option<i32>,
    pub farmers_alerted: Option<i32>,
    pub source: String,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<&Fire> for FireDashboardResponse {
    type Error = AppError;

    fn try_from(fire: &Fire) -> Result<Self, Self::Error> {
        let id = fire.id().ok_or_else(|| {
            AppError::GlobalAppError(GlobalAppError::MissingValue(
                "Fire is missing its id".to_string(),
            ))
        })?;

        Ok(Self {
            id: id.to_string(),
            external_id: fire.external_id().into(),
            lat: fire.location().lat(),
            lon: fire.location().lon(),
            zone_slug: fire.zone_slug().as_ref().map(Into::into),
            place_en: fire.place_en().as_ref().map(Into::into),
            place_ku: fire.place_ku().as_ref().map(Into::into),
            detected_at: *fire.detected_at(),
            area_ha: *fire.area_ha(),
            wind_kmh: *fire.wind_kmh(),
            wind_direction: fire.wind_direction().map(Into::into),
            status: (*fire.status()).into(),
            farms_within_5km: *fire.farms_within_5km(),
            farmers_alerted: *fire.farmers_alerted(),
            source: fire.source().into(),
            updated_at: *fire.updated_at(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FireDashboardOneResponse {
    pub fire: FireDashboardResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FireDashboardListResponse {
    pub fires: Vec<FireDashboardResponse>,
    /// How many fires match in all, not only on this page.
    pub count: u64,
    pub page: u64,
    pub rows_per_page: u64,
}
