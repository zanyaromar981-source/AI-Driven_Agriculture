use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::features::crops::{
    app::{AppError, use_cases::CreateCropInput},
    domain::{self, Crop, CropCode, CropColor, CropDetails, CropName, SortOrder, YieldKgPerDunam},
};

/// The kind of plant, for grouping crops in reports.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CropCategory {
    Cereal,
    Vegetable,
    Fruit,
    Legume,
    Oil,
    Fodder,
    Other,
}

impl From<CropCategory> for domain::CropCategory {
    fn from(value: CropCategory) -> Self {
        match value {
            CropCategory::Cereal => domain::CropCategory::Cereal,
            CropCategory::Vegetable => domain::CropCategory::Vegetable,
            CropCategory::Fruit => domain::CropCategory::Fruit,
            CropCategory::Legume => domain::CropCategory::Legume,
            CropCategory::Oil => domain::CropCategory::Oil,
            CropCategory::Fodder => domain::CropCategory::Fodder,
            CropCategory::Other => domain::CropCategory::Other,
        }
    }
}

impl From<domain::CropCategory> for CropCategory {
    fn from(value: domain::CropCategory) -> Self {
        match value {
            domain::CropCategory::Cereal => CropCategory::Cereal,
            domain::CropCategory::Vegetable => CropCategory::Vegetable,
            domain::CropCategory::Fruit => CropCategory::Fruit,
            domain::CropCategory::Legume => CropCategory::Legume,
            domain::CropCategory::Oil => CropCategory::Oil,
            domain::CropCategory::Fodder => CropCategory::Fodder,
            domain::CropCategory::Other => CropCategory::Other,
        }
    }
}

/// When the crop is in the ground.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CropSeason {
    Winter,
    Summer,
    Perennial,
}

impl From<CropSeason> for domain::CropSeason {
    fn from(value: CropSeason) -> Self {
        match value {
            CropSeason::Winter => domain::CropSeason::Winter,
            CropSeason::Summer => domain::CropSeason::Summer,
            CropSeason::Perennial => domain::CropSeason::Perennial,
        }
    }
}

impl From<domain::CropSeason> for CropSeason {
    fn from(value: domain::CropSeason) -> Self {
        match value {
            domain::CropSeason::Winter => CropSeason::Winter,
            domain::CropSeason::Summer => CropSeason::Summer,
            domain::CropSeason::Perennial => CropSeason::Perennial,
        }
    }
}

/// One crop as stored.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct CropResponse {
    /// The word farms, listings and prices store, for example `wheat`.
    pub code: String,
    pub name_en: String,
    /// Null until the Sorani name has been written.
    pub name_ku: Option<String>,
    /// `#rrggbb`
    pub color: String,
    pub category: CropCategory,
    pub season: CropSeason,
    /// Null = not known.
    pub yield_kg_per_dunam: Option<f64>,
    /// False = no longer offered for new farms, listings and prices.
    pub active: bool,
    /// Lower comes first.
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Crop> for CropResponse {
    fn from(crop: &Crop) -> Self {
        let details = crop.details();

        Self {
            code: crop.code().into(),
            name_en: (&details.name_en).into(),
            name_ku: details.name_ku.as_ref().map(String::from),
            color: (&details.color).into(),
            category: details.category.into(),
            season: details.season.into(),
            yield_kg_per_dunam: details.yield_kg_per_dunam.map(|kg| kg.value()),
            active: details.active,
            sort_order: details.sort_order.value(),
            created_at: *crop.created_at(),
            updated_at: *crop.updated_at(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct CropsResponse {
    pub crops: Vec<CropResponse>,
}

/// The crop after a create or an update.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct OneCropResponse {
    pub crop: CropResponse,
}

impl From<&Crop> for OneCropResponse {
    fn from(crop: &Crop) -> Self {
        Self { crop: crop.into() }
    }
}

fn default_active() -> bool {
    true
}

#[allow(clippy::too_many_arguments)]
fn details(
    name_en: String,
    name_ku: Option<String>,
    color: String,
    category: CropCategory,
    season: CropSeason,
    yield_kg_per_dunam: Option<f64>,
    active: bool,
    sort_order: i32,
) -> Result<CropDetails, AppError> {
    Ok(CropDetails {
        name_en: CropName::new(name_en)?,
        name_ku: name_ku.map(CropName::new).transpose()?,
        color: CropColor::new(color)?,
        category: category.into(),
        season: season.into(),
        yield_kg_per_dunam: yield_kg_per_dunam.map(YieldKgPerDunam::new).transpose()?,
        active,
        sort_order: SortOrder::new(sort_order)?,
    })
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct CreateCropParams {
    /// 2 to 24 lower-case letters and underscores. It never changes, and it
    /// cannot be `empty`.
    pub code: String,
    /// 1 to 60 characters.
    pub name_en: String,
    /// 1 to 60 characters. Leave out or send `null` when not written yet.
    pub name_ku: Option<String>,
    /// `#rrggbb`
    pub color: String,
    pub category: CropCategory,
    pub season: CropSeason,
    /// Above 0, in kg per dunam. Leave out or send `null` when not known.
    pub yield_kg_per_dunam: Option<f64>,
    /// Left out means `true`.
    #[serde(default = "default_active")]
    pub active: bool,
    /// 0 to 100000. Lower comes first.
    pub sort_order: i32,
}

impl CreateCropParams {
    pub fn into_input(self) -> Result<CreateCropInput, AppError> {
        Ok(CreateCropInput {
            code: CropCode::new(self.code)?,
            details: details(
                self.name_en,
                self.name_ku,
                self.color,
                self.category,
                self.season,
                self.yield_kg_per_dunam,
                self.active,
                self.sort_order,
            )?,
        })
    }
}

/// Everything but the code. Every field is replaced: a `name_ku` or a yield
/// left out becomes `null`.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct UpdateCropParams {
    /// 1 to 60 characters.
    pub name_en: String,
    /// 1 to 60 characters, or `null`.
    pub name_ku: Option<String>,
    /// `#rrggbb`
    pub color: String,
    pub category: CropCategory,
    pub season: CropSeason,
    /// Above 0, in kg per dunam, or `null` when not known.
    pub yield_kg_per_dunam: Option<f64>,
    pub active: bool,
    /// 0 to 100000. Lower comes first.
    pub sort_order: i32,
}

impl UpdateCropParams {
    pub fn into_input(self) -> Result<CropDetails, AppError> {
        details(
            self.name_en,
            self.name_ku,
            self.color,
            self.category,
            self.season,
            self.yield_kg_per_dunam,
            self.active,
            self.sort_order,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{app::ToErrorInfo, features::crops::app::testing::a_crop};

    fn create_body(code: &str) -> serde_json::Value {
        serde_json::json!({
            "code": code,
            "name_en": "Rice",
            "color": "#AABBCC",
            "category": "cereal",
            "season": "summer",
            "sort_order": 170
        })
    }

    fn create(body: serde_json::Value) -> Result<CreateCropInput, AppError> {
        serde_json::from_value::<CreateCropParams>(body)
            .expect("params")
            .into_input()
    }

    #[test]
    fn a_crop_serializes_with_the_fields_the_site_reads() {
        let json =
            serde_json::to_value(CropResponse::from(&a_crop("wheat", 10, true))).expect("json");

        assert_eq!(json["code"], "wheat");
        assert_eq!(json["color"], "#e0b13a");
        assert_eq!(json["category"], "cereal");
        assert_eq!(json["season"], "winter");
        assert_eq!(json["active"], true);
        assert_eq!(json["sort_order"], 10);
        assert!(
            json["yield_kg_per_dunam"].is_null() && json["name_ku"].is_null(),
            "what is not known is null, never a guess"
        );
    }

    #[test]
    fn a_create_without_the_optional_fields_is_active_with_no_yield() {
        let input = create(create_body("rice")).expect("input");

        assert_eq!(input.code.as_str(), "rice");
        assert!(input.details.active);
        assert!(input.details.yield_kg_per_dunam.is_none());
        assert!(input.details.name_ku.is_none());
        assert_eq!(input.details.color.as_str(), "#aabbcc");
    }

    #[test]
    fn the_word_for_an_unplanted_cell_is_refused_as_a_code() {
        let error = create(create_body("empty")).err().expect("refused");

        assert_eq!(error.to_error_info().code, "reserved_code");
    }

    #[test]
    fn each_field_is_checked() {
        for (field, bad) in [
            ("code", serde_json::json!("Rice")),
            ("name_en", serde_json::json!("  ")),
            ("name_ku", serde_json::json!("")),
            ("color", serde_json::json!("green")),
            ("yield_kg_per_dunam", serde_json::json!(0)),
            ("sort_order", serde_json::json!(-1)),
        ] {
            let mut body = create_body("rice");
            body[field] = bad;

            assert!(create(body).is_err(), "{field} was accepted");
        }
    }

    #[test]
    fn an_unknown_category_or_season_does_not_parse() {
        for (field, bad) in [("category", "grain"), ("season", "spring")] {
            let mut body = create_body("rice");
            body[field] = serde_json::json!(bad);

            assert!(
                serde_json::from_value::<CreateCropParams>(body).is_err(),
                "{field}"
            );
        }
    }

    #[test]
    fn every_category_and_season_survives_the_trip_through_the_dto_enum() {
        for category in domain::CropCategory::ALL {
            assert_eq!(
                domain::CropCategory::from(CropCategory::from(category)),
                category
            );
            assert_eq!(
                serde_json::to_value(CropCategory::from(category)).expect("json"),
                serde_json::json!(String::from(category)),
                "the wire code and the stored code must be the same word"
            );
        }

        for season in domain::CropSeason::ALL {
            assert_eq!(domain::CropSeason::from(CropSeason::from(season)), season);
            assert_eq!(
                serde_json::to_value(CropSeason::from(season)).expect("json"),
                serde_json::json!(String::from(season)),
                "the wire code and the stored code must be the same word"
            );
        }
    }

    #[test]
    fn an_update_must_say_whether_the_crop_is_active() {
        let body = serde_json::json!({
            "name_en": "Rice",
            "color": "#aabbcc",
            "category": "cereal",
            "season": "summer",
            "sort_order": 170
        });

        assert!(serde_json::from_value::<UpdateCropParams>(body).is_err());
    }
}
