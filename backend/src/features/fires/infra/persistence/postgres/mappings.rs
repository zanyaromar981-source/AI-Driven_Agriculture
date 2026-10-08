use sea_orm::ActiveValue::{NotSet, Set};

use crate::features::fires::{
    app::AppError,
    domain::{
        ExternalId, Fire, FireLocation, FireSource, FireStatus, PlaceName, WindDirection, ZoneSlug,
    },
    infra::persistence::postgres::entities::fires,
};

impl TryFrom<fires::Model> for Fire {
    type Error = AppError;

    fn try_from(model: fires::Model) -> Result<Self, Self::Error> {
        Ok(Fire::rehydrate(
            model.id,
            ExternalId::new(model.external_id)?,
            FireLocation::new(model.lat, model.lon)?,
            model.zone_slug.map(ZoneSlug::new).transpose()?,
            model.place_en.map(PlaceName::new).transpose()?,
            model.place_ku.map(PlaceName::new).transpose()?,
            model.detected_at.and_utc(),
            model.area_ha,
            model.wind_kmh,
            model
                .wind_direction
                .as_deref()
                .map(WindDirection::try_from)
                .transpose()?,
            FireStatus::try_from(model.status.as_str())?,
            model.farms_within_5km,
            model.farmers_alerted,
            FireSource::new(model.source)?,
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&Fire> for fires::ActiveModel {
    fn from(fire: &Fire) -> Self {
        fires::ActiveModel {
            id: match *fire.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            external_id: Set(fire.external_id().into()),
            lat: Set(fire.location().lat()),
            lon: Set(fire.location().lon()),
            zone_slug: Set(fire.zone_slug().as_ref().map(Into::into)),
            place_en: Set(fire.place_en().as_ref().map(Into::into)),
            place_ku: Set(fire.place_ku().as_ref().map(Into::into)),
            detected_at: Set(fire.detected_at().naive_utc()),
            area_ha: Set(*fire.area_ha()),
            wind_kmh: Set(*fire.wind_kmh()),
            wind_direction: Set(fire.wind_direction().map(Into::into)),
            status: Set((*fire.status()).into()),
            farms_within_5km: Set(*fire.farms_within_5km()),
            farmers_alerted: Set(*fire.farmers_alerted()),
            source: Set(fire.source().into()),
            updated_at: Set(fire.updated_at().naive_utc()),
        }
    }
}
