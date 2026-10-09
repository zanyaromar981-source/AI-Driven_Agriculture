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
                ListAllFarmsInput, RegisterFarmInput, RenameFarmInput, RepaintFarmCellsInput,
            },
        },
        domain::{
            self, Cell, CropArea, Farm, FarmName, FarmSummary, GridCell, IdempotencyKey, Outline,
            OwnedFarmSummary, PaintedCell, Point,
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
    /// Every painted cell. Unpainted cells inside the outline become `empty`.
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct CellResponse {
    pub e: i32,
    pub n: i32,
    pub crop: Crop,
}

impl From<&Cell> for CellResponse {
    fn from(cell: &Cell) -> Self {
        Self {
            e: cell.position().e(),
            n: cell.position().n(),
            crop: cell.crop().into(),
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
    pub created_at: DateTime<Utc>,
}

impl From<&FarmSummary> for FarmSummaryResponse {
    fn from(summary: &FarmSummary) -> Self {
        Self {
            id: summary.id().to_string(),
            name: summary.name().into(),
            area_dunam: *summary.area_dunam(),
            crops: summary.crops().iter().map(Into::into).collect(),
            centroid: (*summary.centroid()).into(),
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

        Ok(Self {
            id: id.to_string(),
            name: farm.name().into(),
            area_dunam: farm.area_dunam(),
            crops: farm.crop_areas().iter().map(Into::into).collect(),
            centroid: farm.outline().centroid().into(),
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
/// they fall outside its outline.
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

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DashboardFarmsQuery {
    /// Only the farms of the farmer with exactly this phone, for example
    /// `+9647501234567`.
    pub owner_phone: Option<String>,
}

impl DashboardFarmsQuery {
    pub fn into_input(self, pagination: Pagination) -> Result<ListAllFarmsInput, AppError> {
        Ok(ListAllFarmsInput {
            owner: self
                .owner_phone
                .filter(|phone| !phone.is_empty())
                .map(Phone::new)
                .transpose()?,
            pagination,
        })
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
    use crate::features::farms::app::testing::{OWNER, a_farm};

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
    fn an_empty_phone_filter_is_no_filter_and_a_bad_one_is_refused() {
        let query = |phone: &str| DashboardFarmsQuery {
            owner_phone: Some(phone.to_string()),
        };

        assert!(
            query("")
                .into_input(Pagination::new(1, 20))
                .expect("input")
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
}
