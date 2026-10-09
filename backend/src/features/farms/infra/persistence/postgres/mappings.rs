use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::{NotSet, Set};
use serde::{Deserialize, Serialize};

use crate::{
    app::AppError as GlobalAppError,
    features::farms::{
        app::AppError,
        domain::{
            Cell, Crop, Farm, FarmLocation, FarmName, FarmPlace, FarmSummary, GridCell,
            IdempotencyKey, Outline, Point, UnplacedFarm,
        },
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

pub fn stored_outline(outline: &Outline) -> serde_json::Value {
    let points: Vec<StoredPoint> = outline.points().iter().map(StoredPoint::from).collect();

    serde_json::json!(points)
}

/// The place kept in the farm's three place columns. They are written
/// together, so a row holding only some of them is read as having none.
fn place_from(model: &farms::Model) -> Result<Option<FarmPlace>, AppError> {
    match (&model.governorate, &model.zone_slug, &model.sub_zone_slug) {
        (Some(governorate), Some(zone_slug), Some(sub_zone_slug)) => Ok(Some(FarmPlace::new(
            governorate.clone(),
            zone_slug.clone(),
            sub_zone_slug.clone(),
        )?)),
        _ => Ok(None),
    }
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
            Crop::new(model.crop.as_str())?,
            model.inside_pct,
        ))
    }
}

impl TryFrom<(farms::Model, Vec<farm_cells::Model>)> for Farm {
    type Error = AppError;

    fn try_from(
        (model, cells): (farms::Model, Vec<farm_cells::Model>),
    ) -> Result<Self, Self::Error> {
        let place = place_from(&model)?;

        let mut farm = Farm::rehydrate(
            model.id,
            FarmName::new(model.name)?,
            Phone::new(model.phone)?,
            outline_from(model.outline)?,
            cells
                .into_iter()
                .map(Cell::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            model.idempotency_key.map(IdempotencyKey::new).transpose()?,
            model.created_offline_at.map(|at| at.and_utc()),
            model.created_at.and_utc(),
            model.updated_at.and_utc(),
        );
        farm.place_at(place);

        Ok(farm)
    }
}

impl TryFrom<(farms::Model, Vec<(String, f64)>)> for FarmSummary {
    type Error = AppError;

    fn try_from(
        (model, inside_per_crop): (farms::Model, Vec<(String, f64)>),
    ) -> Result<Self, Self::Error> {
        let inside_per_crop = inside_per_crop
            .into_iter()
            .map(|(crop, inside_pct)| Ok((Crop::new(crop.as_str())?, inside_pct)))
            .collect::<Result<Vec<_>, AppError>>()?;

        let place = place_from(&model)?;

        Ok(FarmSummary::rehydrate(
            model.id,
            FarmName::new(model.name)?,
            &outline_from(model.outline)?,
            place,
            inside_per_crop,
            model.created_at.and_utc(),
        ))
    }
}

impl TryFrom<farms::Model> for FarmLocation {
    type Error = AppError;

    fn try_from(model: farms::Model) -> Result<Self, Self::Error> {
        Ok(FarmLocation::rehydrate(
            model.id,
            &outline_from(model.outline)?,
        ))
    }
}

impl TryFrom<farms::Model> for UnplacedFarm {
    type Error = AppError;

    fn try_from(model: farms::Model) -> Result<Self, Self::Error> {
        Ok(UnplacedFarm::rehydrate(
            model.id,
            outline_from(model.outline)?,
            model.area_m2.is_some(),
            model.updated_at.and_utc(),
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
            idempotency_key: Set(farm.idempotency_key().as_ref().map(Into::into)),
            created_offline_at: Set(farm.created_offline_at().map(|at| at.naive_utc())),
            created_at: Set(farm.created_at().naive_utc()),
            updated_at: Set(farm.updated_at().naive_utc()),
            governorate: Set(farm
                .place()
                .as_ref()
                .map(|place| place.governorate().clone())),
            zone_slug: Set(farm.place().as_ref().map(|place| place.zone_slug().clone())),
            sub_zone_slug: Set(farm
                .place()
                .as_ref()
                .map(|place| place.sub_zone_slug().clone())),
            area_m2: Set(Some(farm.area_m2())),
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
        inside_pct: Set(cell.inside_pct()),
    }
}
