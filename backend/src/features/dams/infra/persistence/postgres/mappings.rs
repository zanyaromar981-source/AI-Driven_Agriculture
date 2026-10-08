use sea_orm::ActiveValue::{NotSet, Set};

use crate::features::dams::{
    app::AppError,
    domain::{Dam, DamReading, DamSlug, PercentFull, ReadingSource},
    infra::persistence::postgres::entities::{dam_readings, dams},
};

impl TryFrom<dams::Model> for Dam {
    type Error = AppError;

    fn try_from(model: dams::Model) -> Result<Self, Self::Error> {
        Ok(Dam::rehydrate(
            model.id,
            DamSlug::new(model.slug)?,
            model.name_en,
            model.name_ku,
            model.capacity_bn_m3,
        ))
    }
}

impl TryFrom<dam_readings::Model> for DamReading {
    type Error = AppError;

    fn try_from(model: dam_readings::Model) -> Result<Self, Self::Error> {
        Ok(DamReading::rehydrate(
            model.id,
            model.dam_id,
            model.day,
            PercentFull::new(model.pct_full)?,
            model.volume_bn_m3,
            model.lake_area_km2,
            model.farm_supply_bn_m3,
            ReadingSource::new(model.source)?,
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&DamReading> for dam_readings::ActiveModel {
    fn from(reading: &DamReading) -> Self {
        dam_readings::ActiveModel {
            id: match *reading.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            dam_id: Set(*reading.dam_id()),
            day: Set(*reading.day()),
            pct_full: Set(reading.pct_full().value()),
            volume_bn_m3: Set(*reading.volume_bn_m3()),
            lake_area_km2: Set(*reading.lake_area_km2()),
            farm_supply_bn_m3: Set(*reading.farm_supply_bn_m3()),
            source: Set(reading.source().into()),
            updated_at: Set(reading.updated_at().naive_utc()),
        }
    }
}
