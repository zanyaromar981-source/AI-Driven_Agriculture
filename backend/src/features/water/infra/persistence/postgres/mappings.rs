use sea_orm::ActiveValue::{NotSet, Set};

use crate::features::water::{
    app::AppError,
    domain::{DamSlug, Need, Note, Season, WaterPlanEntry, ZoneSlug},
    infra::persistence::postgres::entities::water_plan_entries,
};

impl TryFrom<water_plan_entries::Model> for WaterPlanEntry {
    type Error = AppError;

    fn try_from(model: water_plan_entries::Model) -> Result<Self, Self::Error> {
        Ok(WaterPlanEntry::rehydrate(
            model.id,
            Season::new(model.season)?,
            ZoneSlug::new(model.zone_slug)?,
            Need::new(model.need)?,
            model.dam_slug.map(DamSlug::new).transpose()?,
            model.send_million_m3,
            model.urgent,
            model.note_en.map(Note::new).transpose()?,
            model.note_ku.map(Note::new).transpose()?,
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&WaterPlanEntry> for water_plan_entries::ActiveModel {
    fn from(entry: &WaterPlanEntry) -> Self {
        water_plan_entries::ActiveModel {
            id: match *entry.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            season: Set(entry.season().into()),
            zone_slug: Set(entry.zone_slug().into()),
            need: Set(entry.need().value()),
            dam_slug: Set(entry.dam_slug().as_ref().map(Into::into)),
            send_million_m3: Set(*entry.send_million_m3()),
            urgent: Set(*entry.urgent()),
            note_en: Set(entry.note_en().as_ref().map(Into::into)),
            note_ku: Set(entry.note_ku().as_ref().map(Into::into)),
            updated_at: Set(entry.updated_at().naive_utc()),
        }
    }
}
