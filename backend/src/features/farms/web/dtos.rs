use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::farms::{
        app::{
            AppError,
            use_cases::{
                EditFarmInput, ListAllFarmsInput, RegisterFarmInput, RenameFarmInput,
                RepaintFarmCellsInput, ViewFarmStatsInput,
            },
        },
        domain::{
            self, AreaFilter, AreaStats, Cell, CropArea, CropTotals, Farm, FarmFilter, FarmName,
            FarmOrder, FarmPlace, FarmSearch, FarmSortKey, FarmStats, FarmSummary, FarmTotals,
            GridCell, IdempotencyKey, Outline, OwnedFarmSummary, PaintedCell, PlantedCrop, Point,
            SortDirection,
        },
    },
    shared::Phone,
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Crop {
    Wheat,
    Barley,
    Tomato,
    Cucumber,
    Potato,
    Onion,
    Watermelon,
    Grape,
    Olive,
    Sunflower,
    Chickpea,
    Empty,
}

impl From<Crop> for domain::Crop {
    fn from(value: Crop) -> Self {
        match value {
            Crop::Wheat => domain::Crop::Wheat,
            Crop::Barley => domain::Crop::Barley,
            Crop::Tomato => domain::Crop::Tomato,
            Crop::Cucumber => domain::Crop::Cucumber,
            Crop::Potato => domain::Crop::Potato,
            Crop::Onion => domain::Crop::Onion,
            Crop::Watermelon => domain::Crop::Watermelon,
            Crop::Grape => domain::Crop::Grape,
            Crop::Olive => domain::Crop::Olive,
            Crop::Sunflower => domain::Crop::Sunflower,
            Crop::Chickpea => domain::Crop::Chickpea,
            Crop::Empty => domain::Crop::Empty,
        }
    }
}

impl From<domain::Crop> for Crop {
    fn from(value: domain::Crop) -> Self {
        match value {
            domain::Crop::Wheat => Crop::Wheat,
            domain::Crop::Barley => Crop::Barley,
            domain::Crop::Tomato => Crop::Tomato,
            domain::Crop::Cucumber => Crop::Cucumber,
            domain::Crop::Potato => Crop::Potato,
            domain::Crop::Onion => Crop::Onion,
            domain::Crop::Watermelon => Crop::Watermelon,
            domain::Crop::Grape => Crop::Grape,
            domain::Crop::Olive => Crop::Olive,
            domain::Crop::Sunflower => Crop::Sunflower,
            domain::Crop::Chickpea => Crop::Chickpea,
            domain::Crop::Empty => Crop::Empty,
        }
    }
}

/// A GPS corner the farmer tapped: WGS84 decimal degrees, accuracy in metres,
/// UTC time.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct PointParams {
    pub lat: f64,
    pub lon: f64,
    pub acc_m: Option<f64>,
    pub t: Option<DateTime<Utc>>,
}

/// One painted cell of the 10 m grid (UTM zone 38N): `e = floor(easting / 10)`,
/// `n = floor(northing / 10)`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct CellParams {
    pub e: i32,
    pub n: i32,
    pub crop: Crop,
}

impl From<CellParams> for PaintedCell {
    fn from(value: CellParams) -> Self {
        PaintedCell::new(GridCell::new(value.e, value.n), value.crop.into())
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct CreateFarmParams {
    pub name: String,
    /// 3 to 50 corners in walking order. The polygon closes itself.
    pub points: Vec<PointParams>,
    /// Every painted cell. Cells the outline touches that are not listed
    /// become `empty`; listed cells it does not touch come back in
    /// `dropped_cells`.
    #[serde(default)]
    pub cells: Vec<CellParams>,
    pub created_offline_at: Option<DateTime<Utc>>,
}

impl CreateFarmParams {
    pub fn into_input(
        self,
        idempotency_key: Option<String>,
    ) -> Result<RegisterFarmInput, AppError> {
        let points = self
            .points
            .into_iter()
            .map(|point| Point::new(point.lat, point.lon, point.acc_m, point.t))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(RegisterFarmInput {
            name: FarmName::new(self.name)?,
            outline: Outline::new(points)?,
            painted: self.cells.into_iter().map(Into::into).collect(),
            idempotency_key: idempotency_key.map(IdempotencyKey::new).transpose()?,
            created_offline_at: self.created_offline_at,
        })
    }
}

impl CreateFarmParams {
    /// The same body sent to edit a farm. `created_offline_at` is accepted
    /// and not used: it would say when the edit was made on the phone, and
    /// the farm keeps the time it was first drawn.
    pub fn into_edit_input(self) -> Result<EditFarmInput, AppError> {
        let points = self
            .points
            .into_iter()
            .map(|point| Point::new(point.lat, point.lon, point.acc_m, point.t))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(EditFarmInput {
            name: FarmName::new(self.name)?,
            outline: Outline::new(points)?,
            painted: self.cells.into_iter().map(Into::into).collect(),
        })
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct RepaintFarmCellsParams {
    /// The cells to repaint. Cells that are not listed keep their crop.
    pub cells: Vec<CellParams>,
}

impl RepaintFarmCellsParams {
    pub fn into_input(self) -> RepaintFarmCellsInput {
        RepaintFarmCellsInput {
            painted: self.cells.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct CropAreaResponse {
    pub crop: Crop,
    pub dunam: f64,
}

impl From<&CropArea> for CropAreaResponse {
    fn from(area: &CropArea) -> Self {
        Self {
            crop: area.crop().into(),
            dunam: area.dunam(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct CentroidResponse {
    pub lat: f64,
    pub lon: f64,
}

impl From<(f64, f64)> for CentroidResponse {
    fn from((lat, lon): (f64, f64)) -> Self {
        Self { lat, lon }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct OutlinePointResponse {
    pub lat: f64,
    pub lon: f64,
}

impl From<&Point> for OutlinePointResponse {
    fn from(point: &Point) -> Self {
        Self {
            lat: point.lat(),
            lon: point.lon(),
        }
    }
}

/// One cell of a farm. `inside_pct` is the share of the cell's 100 square
/// metres inside the outline, above 0 and at most 100, not rounded: over a
/// farm's cells it adds up to `area_dunam * 2500` square metres.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct CellResponse {
    pub e: i32,
    pub n: i32,
    pub crop: Crop,
    pub inside_pct: f64,
}

impl From<&Cell> for CellResponse {
    fn from(cell: &Cell) -> Self {
        Self {
            e: cell.position().e(),
            n: cell.position().n(),
            crop: cell.crop().into(),
            inside_pct: cell.inside_pct(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct GridCellResponse {
    pub e: i32,
    pub n: i32,
}

impl From<&GridCell> for GridCellResponse {
    fn from(cell: &GridCell) -> Self {
        Self {
            e: cell.e(),
            n: cell.n(),
        }
    }
}

/// The three parts of a farm's place as the answers carry them: all given,
/// or all `null` for a farm outside every sub-district.
fn place_parts(place: &Option<FarmPlace>) -> (Option<String>, Option<String>, Option<String>) {
    match place {
        Some(place) => (
            Some(place.governorate().clone()),
            Some(place.zone_slug().clone()),
            Some(place.sub_zone_slug().clone()),
        ),
        None => (None, None, None),
    }
}

/// The id is an opaque string to the app. `status` and `last_picture` are
/// left out until satellite readings exist; the app treats a missing status
/// as `none`.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FarmSummaryResponse {
    pub id: String,
    pub name: String,
    pub area_dunam: f64,
    pub crops: Vec<CropAreaResponse>,
    pub centroid: CentroidResponse,
    /// The governorate the centroid lies in, by its English name, for
    /// example `Sulaymaniyah`. `null` with the two slugs below when the
    /// farm is outside every sub-district.
    pub governorate: Option<String>,
    /// The district, as `GET /v1/zones/{slug}` names it.
    pub zone_slug: Option<String>,
    /// The sub-district. Unique inside its district only.
    pub sub_zone_slug: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<&FarmSummary> for FarmSummaryResponse {
    fn from(summary: &FarmSummary) -> Self {
        let (governorate, zone_slug, sub_zone_slug) = place_parts(summary.place());

        Self {
            id: summary.id().to_string(),
            name: summary.name().into(),
            area_dunam: *summary.area_dunam(),
            crops: summary.crops().iter().map(Into::into).collect(),
            centroid: (*summary.centroid()).into(),
            governorate,
            zone_slug,
            sub_zone_slug,
            created_at: *summary.created_at(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FarmsResponse {
    pub farms: Vec<FarmSummaryResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FarmResponse {
    pub id: String,
    pub name: String,
    pub area_dunam: f64,
    pub crops: Vec<CropAreaResponse>,
    pub centroid: CentroidResponse,
    /// The place the centroid lies in; see `FarmSummaryResponse`.
    pub governorate: Option<String>,
    pub zone_slug: Option<String>,
    pub sub_zone_slug: Option<String>,
    pub outline: Vec<OutlinePointResponse>,
    pub cells: Vec<CellResponse>,
    pub created_offline_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<&Farm> for FarmResponse {
    type Error = AppError;

    fn try_from(farm: &Farm) -> Result<Self, Self::Error> {
        let id = farm.id().ok_or_else(|| {
            AppError::GlobalAppError(GlobalAppError::MissingValue(
                "Farm is missing its id".to_string(),
            ))
        })?;

        let (governorate, zone_slug, sub_zone_slug) = place_parts(farm.place());

        Ok(Self {
            id: id.to_string(),
            name: farm.name().into(),
            area_dunam: farm.area_dunam(),
            crops: farm.crop_areas().iter().map(Into::into).collect(),
            centroid: farm.outline().centroid().into(),
            governorate,
            zone_slug,
            sub_zone_slug,
            outline: farm.outline().points().iter().map(Into::into).collect(),
            cells: farm.cells().iter().map(Into::into).collect(),
            created_offline_at: *farm.created_offline_at(),
            created_at: *farm.created_at(),
            updated_at: *farm.updated_at(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct OneFarmResponse {
    pub farm: FarmResponse,
}

impl TryFrom<&Farm> for OneFarmResponse {
    type Error = AppError;

    fn try_from(farm: &Farm) -> Result<Self, Self::Error> {
        Ok(Self {
            farm: FarmResponse::try_from(farm)?,
        })
    }
}

/// A farm after a write, with the painted cells that were left out because
/// its outline does not touch them.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct SavedFarmResponse {
    pub farm: FarmResponse,
    pub dropped_cells: Vec<GridCellResponse>,
}

impl TryFrom<(&Farm, &[GridCell])> for SavedFarmResponse {
    type Error = AppError;

    fn try_from((farm, dropped_cells): (&Farm, &[GridCell])) -> Result<Self, Self::Error> {
        Ok(Self {
            farm: FarmResponse::try_from(farm)?,
            dropped_cells: dropped_cells.iter().map(Into::into).collect(),
        })
    }
}

/// One crop plot in the status answer. `greenness_pct_of_normal` stays
/// `null` and `level` stays `none` until a satellite reading exists.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct CropStatusResponse {
    pub crop: Crop,
    pub dunam: f64,
    pub greenness_pct_of_normal: Option<i32>,
    pub level: Level,
}

/// How a cell, a crop or a farm is doing. `none` means no data yet.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Normal,
    Watch,
    Alarm,
    None,
}

/// One cell's reading from space.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct CellStatusResponse {
    pub e: i32,
    pub n: i32,
    pub greenness_pct: Option<i32>,
    pub level: Level,
    pub since: Option<NaiveDate>,
}

/// The farm seen from space. There is no store for satellite readings yet,
/// so every measured field is `null` and `cells` is empty: the app shows
/// "waiting for the first satellite picture". The crop plots are real,
/// taken from the farm's painted cells.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FarmStatusResponse {
    pub picture_date: Option<NaiveDate>,
    pub next_picture_expected: Option<NaiveDate>,
    pub greenness_pct_of_normal: Option<i32>,
    pub weak_where: Option<String>,
    pub cells: Vec<CellStatusResponse>,
    pub crops: Vec<CropStatusResponse>,
}

impl From<&Farm> for FarmStatusResponse {
    fn from(farm: &Farm) -> Self {
        Self {
            picture_date: None,
            next_picture_expected: None,
            greenness_pct_of_normal: None,
            weak_where: None,
            cells: Vec::new(),
            crops: farm
                .crop_areas()
                .iter()
                .map(|area| CropStatusResponse {
                    crop: area.crop().into(),
                    dunam: area.dunam(),
                    greenness_pct_of_normal: None,
                    level: Level::None,
                })
                .collect(),
        }
    }
}

/// A query value that was sent empty (`?zone=`) is one that was not sent.
fn given(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}

fn planted_crop(code: String) -> Result<PlantedCrop, AppError> {
    Ok(PlantedCrop::new(domain::Crop::try_from(code.as_str())?)?)
}

#[derive(Deserialize, Debug, Clone, Default, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DashboardFarmsQuery {
    /// Only the farms of the farmer with exactly this phone, for example
    /// `+9647501234567`.
    pub owner_phone: Option<String>,
    /// Only farms in this governorate, by its name in any case, for example
    /// `Duhok` or `duhok`. `unknown`: only farms with no place.
    pub governorate: Option<String>,
    /// Only farms in the district with this slug. `unknown`: no place.
    pub zone: Option<String>,
    /// Only farms in the sub-district with this slug. `unknown`: no place.
    pub sub_zone: Option<String>,
    /// Only farms with at least one cell of this crop, for example `wheat`.
    /// `empty` is not a crop.
    pub crop: Option<String>,
    /// Only farms whose name or owner phone contains this text, in any
    /// case. `%` and `_` stand for themselves. 1 to 100 characters.
    pub q: Option<String>,
    /// `created_at` (the default), `area_dunam` or `name`.
    pub sort: Option<String>,
    /// `asc` or `desc` (the default). With neither `sort` nor `order` the
    /// newest farm comes first.
    pub order: Option<String>,
}

impl DashboardFarmsQuery {
    pub fn into_input(self, pagination: Pagination) -> Result<ListAllFarmsInput, AppError> {
        Ok(ListAllFarmsInput {
            filter: FarmFilter {
                owner: given(self.owner_phone).map(Phone::new).transpose()?,
                governorate: given(self.governorate).map(AreaFilter::new).transpose()?,
                zone: given(self.zone).map(AreaFilter::new).transpose()?,
                sub_zone: given(self.sub_zone).map(AreaFilter::new).transpose()?,
                crop: given(self.crop).map(planted_crop).transpose()?,
                search: given(self.q).map(FarmSearch::new).transpose()?,
            },
            order: FarmOrder {
                key: given(self.sort)
                    .map(|key| FarmSortKey::try_from(key.as_str()))
                    .transpose()?
                    .unwrap_or_default(),
                direction: given(self.order)
                    .map(|direction| SortDirection::try_from(direction.as_str()))
                    .transpose()?
                    .unwrap_or_default(),
            },
            pagination,
        })
    }
}

#[derive(Deserialize, Debug, Clone, Default, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct FarmStatsQuery {
    /// Only farms in this governorate, by its name in any case. `unknown`:
    /// only farms with no place.
    pub governorate: Option<String>,
    /// Only farms in the district with this slug. `unknown`: no place.
    pub zone: Option<String>,
    /// Only farms growing this crop, and of their crops only this one. The
    /// `dunam` of an area stays the whole area of those farms.
    pub crop: Option<String>,
}

impl FarmStatsQuery {
    pub fn into_input(self) -> Result<ViewFarmStatsInput, AppError> {
        Ok(ViewFarmStatsInput {
            governorate: given(self.governorate).map(AreaFilter::new).transpose()?,
            zone: given(self.zone).map(AreaFilter::new).transpose()?,
            crop: given(self.crop).map(planted_crop).transpose()?,
        })
    }
}

/// Farmers are different phones: one farmer with three farms counts once.
/// `dunam` is the land inside the farms' outlines, painted or not.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct FarmStatsTotalsResponse {
    pub farmers: u64,
    pub farms: u64,
    pub dunam: f64,
}

impl From<&FarmTotals> for FarmStatsTotalsResponse {
    fn from(totals: &FarmTotals) -> Self {
        Self {
            farmers: totals.farmers(),
            farms: totals.farms(),
            dunam: totals.dunam(),
        }
    }
}

/// One crop over the whole answer: the summed share of its cells inside
/// their farms' outlines, and who grows it.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct FarmStatsCropResponse {
    pub crop: Crop,
    pub dunam: f64,
    pub farms: u64,
    pub farmers: u64,
}

impl From<&CropTotals> for FarmStatsCropResponse {
    fn from(crop: &CropTotals) -> Self {
        Self {
            crop: crop.crop().into(),
            dunam: crop.dunam(),
            farms: crop.farms(),
            farmers: crop.farmers(),
        }
    }
}

/// One crop inside one area.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct FarmStatsAreaCropResponse {
    pub crop: Crop,
    pub dunam: f64,
    pub farms: u64,
}

impl From<&CropTotals> for FarmStatsAreaCropResponse {
    fn from(crop: &CropTotals) -> Self {
        Self {
            crop: crop.crop().into(),
            dunam: crop.dunam(),
            farms: crop.farms(),
        }
    }
}

/// The farms of one governorate, district or sub-district. Only areas with
/// at least one farm are listed, north to south. Farms with no place come
/// last as one row with the slug `unknown`. A farmer is counted once in
/// each area where they have a farm.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FarmStatsAreaResponse {
    /// A governorate's slug is its English name in lower case.
    pub slug: String,
    pub name_en: String,
    pub name_ku: String,
    /// On a district and a sub-district: the governorate it lies in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub governorate: Option<String>,
    /// On a sub-district: its district, because a sub-district slug is
    /// unique inside its district only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone_slug: Option<String>,
    pub farmers: u64,
    pub farms: u64,
    /// The whole area of the farms, unpainted land included.
    pub dunam: f64,
    /// Largest first. `empty` is never listed.
    pub crops: Vec<FarmStatsAreaCropResponse>,
}

impl From<&AreaStats> for FarmStatsAreaResponse {
    fn from(area: &AreaStats) -> Self {
        Self {
            slug: area.slug().clone(),
            name_en: area.name_en().clone(),
            name_ku: area.name_ku().clone(),
            governorate: area.governorate().clone(),
            zone_slug: area.zone_slug().clone(),
            farmers: area.totals().farmers(),
            farms: area.totals().farms(),
            dunam: area.totals().dunam(),
            crops: area.crops().iter().map(Into::into).collect(),
        }
    }
}

/// Totals for the dashboard's reports and charts. `as_of` is when a counted
/// farm was last written (the time of asking when none is counted).
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FarmStatsResponse {
    pub as_of: DateTime<Utc>,
    pub totals: FarmStatsTotalsResponse,
    pub by_governorate: Vec<FarmStatsAreaResponse>,
    pub by_zone: Vec<FarmStatsAreaResponse>,
    pub by_sub_zone: Vec<FarmStatsAreaResponse>,
    pub by_crop: Vec<FarmStatsCropResponse>,
}

impl From<&FarmStats> for FarmStatsResponse {
    fn from(stats: &FarmStats) -> Self {
        Self {
            as_of: *stats.as_of(),
            totals: stats.totals().into(),
            by_governorate: stats.by_governorate().iter().map(Into::into).collect(),
            by_zone: stats.by_zone().iter().map(Into::into).collect(),
            by_sub_zone: stats.by_sub_zone().iter().map(Into::into).collect(),
            by_crop: stats.by_crop().iter().map(Into::into).collect(),
        }
    }
}

/// What anyone may read: counts and areas down to districts. No farm, no
/// name, no phone, and nothing per sub-district, where a row could be one
/// farmer.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct PublicFarmStatsResponse {
    pub as_of: DateTime<Utc>,
    pub totals: FarmStatsTotalsResponse,
    pub by_governorate: Vec<FarmStatsAreaResponse>,
    pub by_zone: Vec<FarmStatsAreaResponse>,
    pub by_crop: Vec<FarmStatsCropResponse>,
}

impl From<&FarmStats> for PublicFarmStatsResponse {
    fn from(stats: &FarmStats) -> Self {
        Self {
            as_of: *stats.as_of(),
            totals: stats.totals().into(),
            by_governorate: stats.by_governorate().iter().map(Into::into).collect(),
            by_zone: stats.by_zone().iter().map(Into::into).collect(),
            by_crop: stats.by_crop().iter().map(Into::into).collect(),
        }
    }
}

/// The farmer app's create body, plus the farmer the farm is for.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct DashboardCreateFarmParams {
    /// The phone of a registered farmer, who will own the farm.
    pub owner_phone: String,
    #[serde(flatten)]
    pub farm: CreateFarmParams,
}

impl DashboardCreateFarmParams {
    pub fn into_input(
        self,
        idempotency_key: Option<String>,
    ) -> Result<(Phone, RegisterFarmInput), AppError> {
        Ok((
            Phone::new(self.owner_phone)?,
            self.farm.into_input(idempotency_key)?,
        ))
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct DashboardRenameFarmParams {
    pub name: String,
}

impl DashboardRenameFarmParams {
    pub fn into_input(self) -> Result<RenameFarmInput, AppError> {
        Ok(RenameFarmInput {
            name: FarmName::new(self.name)?,
        })
    }
}

/// A farm's card with its owner's phone. Only staff holding a `farms`
/// permission get this shape.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardFarmSummaryResponse {
    #[serde(flatten)]
    pub summary: FarmSummaryResponse,
    pub owner_phone: String,
}

impl From<&OwnedFarmSummary> for DashboardFarmSummaryResponse {
    fn from(farm: &OwnedFarmSummary) -> Self {
        Self {
            summary: farm.summary().into(),
            owner_phone: farm.owner().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardFarmsResponse {
    pub farms: Vec<DashboardFarmSummaryResponse>,
    /// How many farms match in all, on every page.
    pub count: u64,
    pub page: u64,
    pub rows_per_page: u64,
}

/// A farm in full with its owner's phone.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardFarmResponse {
    #[serde(flatten)]
    pub farm: FarmResponse,
    pub owner_phone: String,
}

impl TryFrom<&Farm> for DashboardFarmResponse {
    type Error = AppError;

    fn try_from(farm: &Farm) -> Result<Self, Self::Error> {
        Ok(Self {
            farm: FarmResponse::try_from(farm)?,
            owner_phone: farm.owner().into(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardOneFarmResponse {
    pub farm: DashboardFarmResponse,
}

impl TryFrom<&Farm> for DashboardOneFarmResponse {
    type Error = AppError;

    fn try_from(farm: &Farm) -> Result<Self, Self::Error> {
        Ok(Self {
            farm: DashboardFarmResponse::try_from(farm)?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardSavedFarmResponse {
    pub farm: DashboardFarmResponse,
    pub dropped_cells: Vec<GridCellResponse>,
}

impl TryFrom<(&Farm, &[GridCell])> for DashboardSavedFarmResponse {
    type Error = AppError;

    fn try_from((farm, dropped_cells): (&Farm, &[GridCell])) -> Result<Self, Self::Error> {
        Ok(Self {
            farm: DashboardFarmResponse::try_from(farm)?,
            dropped_cells: dropped_cells.iter().map(Into::into).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::app::testing::{OWNER, a_farm, the_place};

    #[test]
    fn the_app_shape_of_a_farm_never_carries_the_owners_phone() {
        let body = serde_json::to_string(&OneFarmResponse::try_from(&a_farm()).expect("farm"))
            .expect("json");

        assert!(
            !body.contains(OWNER) && !body.contains("owner_phone"),
            "the phone is shown on the dashboard only"
        );
    }

    #[test]
    fn the_dashboard_shape_is_the_app_shape_plus_the_owners_phone() {
        let farm = a_farm();
        let app = serde_json::to_value(FarmResponse::try_from(&farm).expect("farm")).expect("json");
        let mut dashboard =
            serde_json::to_value(DashboardFarmResponse::try_from(&farm).expect("farm"))
                .expect("json");

        assert_eq!(dashboard["owner_phone"], OWNER);

        dashboard
            .as_object_mut()
            .expect("object")
            .remove("owner_phone");
        assert_eq!(dashboard, app);
    }

    #[test]
    fn every_cell_carries_inside_pct_in_the_app_shape_and_the_dashboard_shape() {
        let farm = a_farm();
        let app =
            serde_json::to_value(OneFarmResponse::try_from(&farm).expect("farm")).expect("json");
        let dashboard =
            serde_json::to_value(DashboardOneFarmResponse::try_from(&farm).expect("farm"))
                .expect("json");

        for body in [app, dashboard] {
            let cells = body["farm"]["cells"].as_array().expect("cells").clone();

            assert_eq!(cells.len(), farm.cells().len());
            assert!(cells.iter().zip(farm.cells()).all(|(cell, stored)| {
                cell["inside_pct"].as_f64() == Some(stored.inside_pct())
            }));
        }
    }

    #[test]
    fn the_edit_body_is_the_create_body_and_is_checked_the_same_way() {
        let body = |points: serde_json::Value| -> CreateFarmParams {
            serde_json::from_value(serde_json::json!({
                "name": "Lower field",
                "points": points,
                "cells": [{"e": 46_415, "n": 398_748, "crop": "wheat"}],
                "created_offline_at": "2026-10-08T14:10:00Z"
            }))
            .expect("body")
        };

        let input = body(serde_json::json!([
            {"lat": 36.0300, "lon": 44.6000},
            {"lat": 36.0300, "lon": 44.6010},
            {"lat": 36.0310, "lon": 44.6010}
        ]))
        .into_edit_input()
        .expect("input");

        assert_eq!(input.name.as_str(), "Lower field");
        assert_eq!(input.painted.len(), 1);

        let crossing = body(serde_json::json!([
            {"lat": 36.0300, "lon": 44.6000},
            {"lat": 36.0300, "lon": 44.6010},
            {"lat": 36.0310, "lon": 44.6000},
            {"lat": 36.0310, "lon": 44.6010}
        ]))
        .into_edit_input();

        assert!(matches!(
            crossing,
            Err(AppError::Farm(domain::FarmError::OutlineSelfIntersects))
        ));
    }

    #[test]
    fn an_empty_phone_filter_is_no_filter_and_a_bad_one_is_refused() {
        let query = |phone: &str| DashboardFarmsQuery {
            owner_phone: Some(phone.to_string()),
            ..DashboardFarmsQuery::default()
        };

        assert!(
            query("")
                .into_input(Pagination::new(1, 20))
                .expect("input")
                .filter
                .owner
                .is_none()
        );
        assert!(query("0750").into_input(Pagination::new(1, 20)).is_err());
    }

    #[test]
    fn the_staff_create_body_is_the_app_body_plus_the_owner() {
        let params: DashboardCreateFarmParams = serde_json::from_value(serde_json::json!({
            "owner_phone": OWNER,
            "name": "Upper field",
            "points": [
                {"lat": 36.0300, "lon": 44.6000},
                {"lat": 36.0300, "lon": 44.6010},
                {"lat": 36.0310, "lon": 44.6010}
            ]
        }))
        .expect("body");

        let (owner, input) = params.into_input(None).expect("input");

        assert_eq!(owner.as_str(), OWNER);
        assert_eq!(input.name.as_str(), "Upper field");
    }

    fn placed(farm: &Farm) -> Farm {
        let mut farm = farm.clone();
        farm.place_at(Some(the_place()));
        farm
    }

    #[test]
    fn a_farm_inside_a_sub_district_carries_its_place_in_every_shape() {
        let farm = placed(&a_farm());
        let app =
            serde_json::to_value(OneFarmResponse::try_from(&farm).expect("farm")).expect("json");
        let dashboard =
            serde_json::to_value(DashboardOneFarmResponse::try_from(&farm).expect("farm"))
                .expect("json");

        for body in [app, dashboard] {
            assert_eq!(body["farm"]["governorate"], "Sulaymaniyah");
            assert_eq!(body["farm"]["zone_slug"], "chamchamal");
            assert_eq!(body["farm"]["sub_zone_slug"], "sangaw");
        }
    }

    #[test]
    fn a_farm_outside_every_sub_district_carries_three_nulls_not_missing_fields() {
        let body = serde_json::to_value(OneFarmResponse::try_from(&a_farm()).expect("farm"))
            .expect("json");
        let farm = body["farm"].as_object().expect("object");

        for field in ["governorate", "zone_slug", "sub_zone_slug"] {
            assert_eq!(farm.get(field), Some(&serde_json::Value::Null), "{field}");
        }
    }

    #[test]
    fn the_farm_card_carries_the_place_for_the_app_and_for_the_dashboard() {
        let summary = FarmSummary::rehydrate(
            7,
            FarmName::new("Upper field".to_string()).expect("name"),
            a_farm().outline(),
            Some(the_place()),
            vec![],
            Utc::now(),
        );
        let owned = OwnedFarmSummary::new(Phone::new(OWNER.to_string()).expect("phone"), summary);

        let app = serde_json::to_value(FarmSummaryResponse::from(owned.summary())).expect("json");
        let dashboard =
            serde_json::to_value(DashboardFarmSummaryResponse::from(&owned)).expect("json");

        for body in [&app, &dashboard] {
            assert_eq!(body["governorate"], "Sulaymaniyah");
            assert_eq!(body["zone_slug"], "chamchamal");
            assert_eq!(body["sub_zone_slug"], "sangaw");
        }
        assert!(app.get("owner_phone").is_none());
    }

    fn farms_query(json: serde_json::Value) -> Result<ListAllFarmsInput, AppError> {
        serde_json::from_value::<DashboardFarmsQuery>(json)
            .expect("query")
            .into_input(Pagination::new(1, 20))
    }

    #[test]
    fn with_no_query_every_farm_is_listed_newest_first() {
        let input = farms_query(serde_json::json!({})).expect("input");

        assert_eq!(input.filter, FarmFilter::default());
        assert_eq!(input.order, FarmOrder::default());
    }

    #[test]
    fn every_filter_and_the_order_are_read_from_the_query() {
        let input = farms_query(serde_json::json!({
            "governorate": "Duhok",
            "zone": "zakho",
            "sub_zone": "unknown",
            "crop": "wheat",
            "q": " upper ",
            "sort": "area_dunam",
            "order": "asc"
        }))
        .expect("input");

        assert_eq!(
            input.filter.governorate,
            Some(AreaFilter::Named("duhok".to_string()))
        );
        assert_eq!(
            input.filter.zone,
            Some(AreaFilter::Named("zakho".to_string()))
        );
        assert_eq!(input.filter.sub_zone, Some(AreaFilter::Unknown));
        assert_eq!(
            input.filter.crop.map(|crop| crop.crop()),
            Some(domain::Crop::Wheat)
        );
        assert_eq!(
            input.filter.search.as_ref().map(FarmSearch::as_str),
            Some("upper")
        );
        assert_eq!(input.order.key, FarmSortKey::AreaDunam);
        assert_eq!(input.order.direction, SortDirection::Ascending);
    }

    #[test]
    fn a_filter_sent_empty_is_a_filter_not_sent() {
        let input = farms_query(serde_json::json!({
            "governorate": "", "zone": " ", "sub_zone": "", "crop": "", "q": "", "sort": "",
            "order": ""
        }))
        .expect("input");

        assert_eq!(input.filter, FarmFilter::default());
        assert_eq!(input.order, FarmOrder::default());
    }

    #[test]
    fn an_unknown_crop_sort_or_order_and_an_over_long_search_are_refused() {
        for query in [
            serde_json::json!({"crop": "rice"}),
            serde_json::json!({"crop": "empty"}),
            serde_json::json!({"sort": "owner_phone"}),
            serde_json::json!({"order": "down"}),
            serde_json::json!({"q": "x".repeat(101)}),
            serde_json::json!({"zone": "z".repeat(61)}),
        ] {
            assert!(farms_query(query.clone()).is_err(), "{query}");
        }
    }

    #[test]
    fn the_stats_query_takes_a_governorate_a_zone_and_a_crop() {
        let input = serde_json::from_value::<FarmStatsQuery>(serde_json::json!({
            "governorate": "Erbil", "zone": "", "crop": "barley"
        }))
        .expect("query")
        .into_input()
        .expect("input");

        assert_eq!(
            input.governorate,
            Some(AreaFilter::Named("erbil".to_string()))
        );
        assert_eq!(input.zone, None);
        assert_eq!(
            input.crop.map(|crop| crop.crop()),
            Some(domain::Crop::Barley)
        );

        let empty_land = FarmStatsQuery {
            crop: Some("empty".to_string()),
            ..FarmStatsQuery::default()
        };
        assert!(empty_land.into_input().is_err());
    }

    /// One farm in Sangaw with six dunams of wheat, and one with no place.
    fn some_stats() -> FarmStats {
        use domain::{AreaCount, AreaCropSum, AreaKey, AreaLevel};

        let sangaw = |sub_zone: Option<&str>, zone: Option<&str>| AreaKey {
            governorate: Some("Sulaymaniyah".to_string()),
            zone_slug: zone.map(str::to_string),
            sub_zone_slug: sub_zone.map(str::to_string),
        };
        let count = |level, key, farms| AreaCount {
            level,
            key,
            farms,
            farmers: farms,
            area_m2: 25_000.0 * farms as f64,
            latest_change: None,
        };
        let wheat = |level, key| AreaCropSum {
            level,
            key,
            crop: domain::Crop::Wheat,
            inside_pct: 15_000.0,
            farms: 1,
            farmers: 1,
        };

        FarmStats::assemble(
            vec![
                count(AreaLevel::Region, AreaKey::default(), 2),
                count(AreaLevel::Governorate, sangaw(None, None), 1),
                count(AreaLevel::Governorate, AreaKey::default(), 1),
                count(AreaLevel::Zone, sangaw(None, Some("chamchamal")), 1),
                count(AreaLevel::Zone, AreaKey::default(), 1),
                count(
                    AreaLevel::SubZone,
                    sangaw(Some("sangaw"), Some("chamchamal")),
                    1,
                ),
                count(AreaLevel::SubZone, AreaKey::default(), 1),
            ],
            vec![
                wheat(AreaLevel::Region, AreaKey::default()),
                wheat(AreaLevel::Zone, sangaw(None, Some("chamchamal"))),
            ],
            &crate::features::farms::app::testing::area_names(),
            Utc::now(),
        )
    }

    #[test]
    fn the_dashboard_stats_have_the_shape_the_website_asked_for() {
        let body = serde_json::to_value(FarmStatsResponse::from(&some_stats())).expect("json");

        assert_eq!(
            body["totals"],
            serde_json::json!({"farmers": 2, "farms": 2, "dunam": 20.0})
        );
        assert_eq!(
            body["by_crop"],
            serde_json::json!([{"crop": "wheat", "dunam": 6.0, "farms": 1, "farmers": 1}])
        );
        assert_eq!(
            body["by_zone"][0],
            serde_json::json!({
                "slug": "chamchamal",
                "name_en": "Chamchamal",
                "name_ku": "چەمچەماڵ",
                "governorate": "Sulaymaniyah",
                "farmers": 1,
                "farms": 1,
                "dunam": 10.0,
                "crops": [{"crop": "wheat", "dunam": 6.0, "farms": 1}]
            })
        );
        assert_eq!(body["by_governorate"][0]["slug"], "sulaymaniyah");
        assert_eq!(body["by_governorate"][1]["slug"], "unknown");
        assert_eq!(body["by_sub_zone"][0]["zone_slug"], "chamchamal");
        assert!(body["as_of"].is_string());
    }

    #[test]
    fn the_public_stats_carry_no_sub_zone_and_nothing_about_a_person() {
        let body =
            serde_json::to_string(&PublicFarmStatsResponse::from(&some_stats())).expect("json");
        let json: serde_json::Value = serde_json::from_str(&body).expect("json");

        let mut keys: Vec<&str> = json
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();

        assert_eq!(
            keys,
            vec!["as_of", "by_crop", "by_governorate", "by_zone", "totals"]
        );
        for hidden in ["sub_zone", "sangaw", "phone", "+964", "owner", "\"name\""] {
            assert!(!body.contains(hidden), "{hidden} must not be public");
        }
    }
}
