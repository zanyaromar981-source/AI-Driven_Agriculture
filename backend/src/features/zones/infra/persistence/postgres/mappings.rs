use sea_orm::{
    ActiveValue::{NotSet, Set},
    EntityTrait, FromQueryResult, QuerySelect, Select,
};

use crate::{
    app::AppError as GlobalAppError,
    features::zones::{
        app::AppError,
        domain::{
            Crop, Dryness, GreennessPctVsNormal, Month, RainPctOfNormal, ReadingSource, Shape,
            SubZone, SubZoneReading, WaterNeed, Zone, ZoneReading, ZoneSlug,
        },
        infra::persistence::postgres::entities::{
            sub_zone_readings, sub_zones, zone_readings, zones,
        },
    },
};

/// The best crops are kept in the `best_crops` JSON column as an array of
/// crop codes, best first.
fn stored_crops(crops: &[Crop]) -> serde_json::Value {
    let codes: Vec<String> = crops.iter().map(|crop| String::from(*crop)).collect();

    serde_json::json!(codes)
}

fn crops_from(stored: serde_json::Value) -> Result<Vec<Crop>, AppError> {
    let codes: Vec<String> = serde_json::from_value(stored).map_err(|error| {
        GlobalAppError::MissingValue(format!("Stored best crops are not readable: {error}"))
    })?;

    codes
        .iter()
        .map(|code| Ok(Crop::try_from(code.as_str())?))
        .collect()
}

impl TryFrom<zones::Model> for Zone {
    type Error = AppError;

    fn try_from(model: zones::Model) -> Result<Self, Self::Error> {
        Ok(Zone::rehydrate(
            model.id,
            ZoneSlug::new(model.slug)?,
            model.name_en,
            model.name_ku,
            model.governorate,
        ))
    }
}

/// A sub-zone as every query but the shapes one reads it: without its
/// outline, which runs to hundreds of corners and is wanted only to build
/// the place index.
#[derive(FromQueryResult)]
pub struct SubZoneRow {
    pub id: i32,
    pub zone_id: i32,
    pub slug: String,
    pub name_en: String,
    pub name_ku: String,
}

impl SubZoneRow {
    pub fn select() -> Select<sub_zones::Entity> {
        sub_zones::Entity::find().select_only().columns([
            sub_zones::Column::Id,
            sub_zones::Column::ZoneId,
            sub_zones::Column::Slug,
            sub_zones::Column::NameEn,
            sub_zones::Column::NameKu,
        ])
    }
}

impl TryFrom<SubZoneRow> for SubZone {
    type Error = AppError;

    fn try_from(row: SubZoneRow) -> Result<Self, Self::Error> {
        Ok(SubZone::rehydrate(
            row.id,
            row.zone_id,
            ZoneSlug::new(row.slug)?,
            row.name_en,
            row.name_ku,
        ))
    }
}

/// A sub-zone with the shape kept in its `outline` JSON column: an array of
/// rings, each an array of `[lon, lat]`.
pub fn shaped_sub_zone(
    model: sub_zones::Model,
    outline: serde_json::Value,
) -> Result<(SubZone, Shape), AppError> {
    let rings: Vec<Vec<(f64, f64)>> = serde_json::from_value(outline).map_err(|error| {
        GlobalAppError::MissingValue(format!(
            "Stored outline of sub-zone {} is not readable: {error}",
            model.slug
        ))
    })?;

    Ok((
        SubZone::rehydrate(
            model.id,
            model.zone_id,
            ZoneSlug::new(model.slug)?,
            model.name_en,
            model.name_ku,
        ),
        Shape::new(rings)?,
    ))
}

impl TryFrom<zone_readings::Model> for ZoneReading {
    type Error = AppError;

    fn try_from(model: zone_readings::Model) -> Result<Self, Self::Error> {
        Ok(ZoneReading::rehydrate(
            model.id,
            model.zone_id,
            Month::containing(model.month),
            Dryness::new(model.dryness)?,
            model
                .rain_pct_of_normal
                .map(RainPctOfNormal::new)
                .transpose()?,
            model
                .greenness_pct_vs_normal
                .map(GreennessPctVsNormal::new)
                .transpose()?,
            model.water_need.map(WaterNeed::new).transpose()?,
            model.nitrogen_hold,
            crops_from(model.best_crops)?,
            ReadingSource::new(model.source)?,
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&ZoneReading> for zone_readings::ActiveModel {
    fn from(reading: &ZoneReading) -> Self {
        zone_readings::ActiveModel {
            id: match *reading.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            zone_id: Set(*reading.zone_id()),
            month: Set(reading.month().first_day()),
            dryness: Set(reading.dryness().value()),
            rain_pct_of_normal: Set(reading.rain_pct_of_normal().map(|rain| rain.value())),
            greenness_pct_vs_normal: Set(reading
                .greenness_pct_vs_normal()
                .map(|greenness| greenness.value())),
            water_need: Set(reading.water_need().map(|need| need.value())),
            nitrogen_hold: Set(*reading.nitrogen_hold()),
            best_crops: Set(stored_crops(reading.best_crops())),
            source: Set(reading.source().into()),
            updated_at: Set(reading.updated_at().naive_utc()),
        }
    }
}

impl TryFrom<sub_zone_readings::Model> for SubZoneReading {
    type Error = AppError;

    fn try_from(model: sub_zone_readings::Model) -> Result<Self, Self::Error> {
        Ok(SubZoneReading::rehydrate(
            model.id,
            model.sub_zone_id,
            Month::containing(model.month),
            Dryness::new(model.dryness)?,
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&SubZoneReading> for sub_zone_readings::ActiveModel {
    fn from(reading: &SubZoneReading) -> Self {
        sub_zone_readings::ActiveModel {
            id: match *reading.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            sub_zone_id: Set(*reading.sub_zone_id()),
            month: Set(reading.month().first_day()),
            dryness: Set(reading.dryness().value()),
            updated_at: Set(reading.updated_at().naive_utc()),
        }
    }
}
