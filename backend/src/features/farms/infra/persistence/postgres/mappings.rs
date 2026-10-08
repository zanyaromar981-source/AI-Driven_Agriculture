use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::{NotSet, Set};
use serde::{Deserialize, Serialize};

use crate::{
    app::AppError as GlobalAppError,
    features::farms::{
        app::AppError,
        domain::{Cell, Crop, Farm, FarmName, FarmSummary, GridCell, Outline, Point},
        infra::persistence::postgres::entities::{farm_cells, farms},
    },
    shared::Phone,
};

/// One corner of the outline as it is kept in the `outline` JSON column.
#[derive(Serialize, Deserialize)]
struct StoredPoint {
    lat: f64,
    lon: f64,
    acc_m: Option<f64>,
    t: Option<DateTime<Utc>>,
}

impl From<&Point> for StoredPoint {
    fn from(point: &Point) -> Self {
        Self {
            lat: point.lat(),
            lon: point.lon(),
            acc_m: point.accuracy_m(),
            t: point.recorded_at(),
        }
    }
}

fn stored_outline(outline: &Outline) -> serde_json::Value {
    let points: Vec<StoredPoint> = outline.points().iter().map(StoredPoint::from).collect();

    serde_json::json!(points)
}

fn outline_from(stored: serde_json::Value) -> Result<Outline, AppError> {
    let points: Vec<StoredPoint> = serde_json::from_value(stored).map_err(|error| {
        GlobalAppError::MissingValue(format!("Stored farm outline is not readable: {error}"))
    })?;

    let points = points
        .into_iter()
        .map(|point| Point::new(point.lat, point.lon, point.acc_m, point.t))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Outline::new(points)?)
}

impl TryFrom<farm_cells::Model> for Cell {
    type Error = AppError;

    fn try_from(model: farm_cells::Model) -> Result<Self, Self::Error> {
        Ok(Cell::rehydrate(
            model.id,
            GridCell::new(model.e, model.n),
            Crop::try_from(model.crop.as_str())?,
        ))
    }
}

impl TryFrom<(farms::Model, Vec<farm_cells::Model>)> for Farm {
    type Error = AppError;

    fn try_from(
        (model, cells): (farms::Model, Vec<farm_cells::Model>),
    ) -> Result<Self, Self::Error> {
        Ok(Farm::rehydrate(
            model.id,
            FarmName::new(model.name)?,
            Phone::new(model.phone)?,
            outline_from(model.outline)?,
            cells
                .into_iter()
                .map(Cell::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            model.created_offline_at.map(|at| at.and_utc()),
            model.created_at.and_utc(),
            model.updated_at.and_utc(),
        ))
    }
}

impl TryFrom<(farms::Model, Vec<(String, usize)>)> for FarmSummary {
    type Error = AppError;

    fn try_from(
        (model, cells_per_crop): (farms::Model, Vec<(String, usize)>),
    ) -> Result<Self, Self::Error> {
        let cells_per_crop = cells_per_crop
            .into_iter()
            .map(|(crop, cells)| Ok((Crop::try_from(crop.as_str())?, cells)))
            .collect::<Result<Vec<_>, AppError>>()?;

        Ok(FarmSummary::rehydrate(
            model.id,
            FarmName::new(model.name)?,
            &outline_from(model.outline)?,
            cells_per_crop,
            model.created_at.and_utc(),
        ))
    }
}

impl From<&Farm> for farms::ActiveModel {
    fn from(farm: &Farm) -> Self {
        farms::ActiveModel {
            id: match *farm.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            name: Set(farm.name().into()),
            phone: Set(farm.owner().into()),
            outline: Set(stored_outline(farm.outline())),
            created_offline_at: Set(farm.created_offline_at().map(|at| at.naive_utc())),
            created_at: Set(farm.created_at().naive_utc()),
            updated_at: Set(farm.updated_at().naive_utc()),
        }
    }
}

pub fn cell_active_model(farm_id: i32, cell: &Cell) -> farm_cells::ActiveModel {
    farm_cells::ActiveModel {
        id: match cell.id() {
            Some(id) => Set(id),
            None => NotSet,
        },
        farm_id: Set(farm_id),
        e: Set(cell.position().e()),
        n: Set(cell.position().n()),
        crop: Set(cell.crop().into()),
    }
}
