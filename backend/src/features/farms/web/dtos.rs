use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    app::AppError as GlobalAppError,
    features::farms::{
        app::{
            AppError,
            use_cases::{RegisterFarmInput, RepaintFarmCellsInput},
        },
        domain::{
            self, Cell, CropArea, Farm, FarmName, FarmSummary, GridCell, Outline, PaintedCell,
            Point,
        },
    },
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
    pub fn into_input(self) -> Result<RegisterFarmInput, AppError> {
        let points = self
            .points
            .into_iter()
            .map(|point| Point::new(point.lat, point.lon, point.acc_m, point.t))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(RegisterFarmInput {
            name: FarmName::new(self.name)?,
            outline: Outline::new(points)?,
            painted: self.cells.into_iter().map(Into::into).collect(),
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

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FarmSummaryResponse {
    pub id: i32,
    pub name: String,
    pub area_dunam: f64,
    pub crops: Vec<CropAreaResponse>,
    pub centroid: CentroidResponse,
    pub created_at: NaiveDateTime,
}

impl From<&FarmSummary> for FarmSummaryResponse {
    fn from(summary: &FarmSummary) -> Self {
        Self {
            id: *summary.id(),
            name: summary.name().into(),
            area_dunam: *summary.area_dunam(),
            crops: summary.crops().iter().map(Into::into).collect(),
            centroid: (*summary.centroid()).into(),
            created_at: summary.created_at().naive_utc(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FarmResponse {
    pub id: i32,
    pub name: String,
    pub area_dunam: f64,
    pub crops: Vec<CropAreaResponse>,
    pub centroid: CentroidResponse,
    pub outline: Vec<OutlinePointResponse>,
    pub cells: Vec<CellResponse>,
    pub created_offline_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
            id,
            name: farm.name().into(),
            area_dunam: farm.area_dunam(),
            crops: farm.crop_areas().iter().map(Into::into).collect(),
            centroid: farm.outline().centroid().into(),
            outline: farm.outline().points().iter().map(Into::into).collect(),
            cells: farm.cells().iter().map(Into::into).collect(),
            created_offline_at: farm.created_offline_at().map(|at| at.naive_utc()),
            created_at: farm.created_at().naive_utc(),
            updated_at: farm.updated_at().naive_utc(),
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
