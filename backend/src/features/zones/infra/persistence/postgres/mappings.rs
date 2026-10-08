use sea_orm::ActiveValue::{NotSet, Set};

use crate::{
    app::AppError as GlobalAppError,
    features::zones::{
        app::AppError,
        domain::{
            Crop, Dryness, GreennessPctVsNormal, Month, RainPctOfNormal, ReadingSource, SubZone,
            SubZoneReading, WaterNeed, Zone, ZoneReading, ZoneSlug,
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

impl TryFrom<sub_zones::Model> for SubZone {
    type Error = AppError;

    fn try_from(model: sub_zones::Model) -> Result<Self, Self::Error> {
        Ok(SubZone::rehydrate(
            model.id,
            model.zone_id,
            ZoneSlug::new(model.slug)?,
            model.name_en,
            model.name_ku,
        ))
    }
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
