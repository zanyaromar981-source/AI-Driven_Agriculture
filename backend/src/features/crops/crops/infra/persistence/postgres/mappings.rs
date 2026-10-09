use sea_orm::ActiveValue::Set;

use crate::features::crops::{
    app::AppError,
    domain::{
        Crop, CropCategory, CropCode, CropColor, CropDetails, CropName, CropSeason, SortOrder,
        YieldKgPerDunam,
    },
    infra::persistence::postgres::entities::crops,
};

impl TryFrom<crops::Model> for Crop {
    type Error = AppError;

    fn try_from(model: crops::Model) -> Result<Self, Self::Error> {
        Ok(Crop::rehydrate(
            CropCode::new(model.code)?,
            CropDetails {
                name_en: CropName::new(model.name_en)?,
                name_ku: model.name_ku.map(CropName::new).transpose()?,
                color: CropColor::new(model.color)?,
                category: CropCategory::try_from(model.category.as_str())?,
                season: CropSeason::try_from(model.season.as_str())?,
                yield_kg_per_dunam: model
                    .yield_kg_per_dunam
                    .map(YieldKgPerDunam::new)
                    .transpose()?,
                active: model.active,
                sort_order: SortOrder::new(model.sort_order)?,
            },
            model.created_at.and_utc(),
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&Crop> for crops::ActiveModel {
    fn from(crop: &Crop) -> Self {
        let details = crop.details();

        crops::ActiveModel {
            code: Set(crop.code().into()),
            name_en: Set((&details.name_en).into()),
            name_ku: Set(details.name_ku.as_ref().map(String::from)),
            color: Set((&details.color).into()),
            category: Set(details.category.into()),
            season: Set(details.season.into()),
            yield_kg_per_dunam: Set(details.yield_kg_per_dunam.map(|kg| kg.value())),
            active: Set(details.active),
            sort_order: Set(details.sort_order.value()),
            created_at: Set(crop.created_at().naive_utc()),
            updated_at: Set(crop.updated_at().naive_utc()),
        }
    }
}
