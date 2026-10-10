use crate::{features::crops::domain::CropError, shared::DomainError};

/// The kind of plant, for grouping crops in reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CropCategory {
    Cereal,
    Vegetable,
    Fruit,
    Legume,
    Oil,
    Fodder,
    Other,
}

impl CropCategory {
    pub const ALL: [CropCategory; 7] = [
        CropCategory::Cereal,
        CropCategory::Vegetable,
        CropCategory::Fruit,
        CropCategory::Legume,
        CropCategory::Oil,
        CropCategory::Fodder,
        CropCategory::Other,
    ];
}

impl From<CropCategory> for String {
    fn from(value: CropCategory) -> Self {
        match value {
            CropCategory::Cereal => "cereal".to_string(),
            CropCategory::Vegetable => "vegetable".to_string(),
            CropCategory::Fruit => "fruit".to_string(),
            CropCategory::Legume => "legume".to_string(),
            CropCategory::Oil => "oil".to_string(),
            CropCategory::Fodder => "fodder".to_string(),
            CropCategory::Other => "other".to_string(),
        }
    }
}

impl TryFrom<&str> for CropCategory {
    type Error = CropError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "cereal" => Ok(CropCategory::Cereal),
            "vegetable" => Ok(CropCategory::Vegetable),
            "fruit" => Ok(CropCategory::Fruit),
            "legume" => Ok(CropCategory::Legume),
            "oil" => Ok(CropCategory::Oil),
            "fodder" => Ok(CropCategory::Fodder),
            "other" => Ok(CropCategory::Other),
            _ => Err(DomainError::InvalidValue(format!("Invalid crop category: {value}")).into()),
        }
    }
}

/// When the crop is in the ground: sown for the winter rains, grown through
/// the summer, or standing all year like a tree or a vine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CropSeason {
    Winter,
    Summer,
    Perennial,
}

impl CropSeason {
    pub const ALL: [CropSeason; 3] = [
        CropSeason::Winter,
        CropSeason::Summer,
        CropSeason::Perennial,
    ];
}

impl From<CropSeason> for String {
    fn from(value: CropSeason) -> Self {
        match value {
            CropSeason::Winter => "winter".to_string(),
            CropSeason::Summer => "summer".to_string(),
            CropSeason::Perennial => "perennial".to_string(),
        }
    }
}

impl TryFrom<&str> for CropSeason {
    type Error = CropError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "winter" => Ok(CropSeason::Winter),
            "summer" => Ok(CropSeason::Summer),
            "perennial" => Ok(CropSeason::Perennial),
            _ => Err(DomainError::InvalidValue(format!("Invalid crop season: {value}")).into()),
        }
    }
}

/// Which shelf of the Marketplace a product stands on. Only `Crops` grow in
/// a field, so only they can be painted on a farm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProductGroup {
    // In display order: the derived order is the order of the shelves.
    Crops,
    FishMeatEggs,
    HoneyDairy,
    Animals,
    NutsDried,
}

impl ProductGroup {
    pub const ALL: [ProductGroup; 5] = [
        ProductGroup::Crops,
        ProductGroup::FishMeatEggs,
        ProductGroup::HoneyDairy,
        ProductGroup::Animals,
        ProductGroup::NutsDried,
    ];
}

impl From<ProductGroup> for String {
    fn from(value: ProductGroup) -> Self {
        match value {
            ProductGroup::Crops => "crops".to_string(),
            ProductGroup::FishMeatEggs => "fish_meat_eggs".to_string(),
            ProductGroup::HoneyDairy => "honey_dairy".to_string(),
            ProductGroup::Animals => "animals".to_string(),
            ProductGroup::NutsDried => "nuts_dried".to_string(),
        }
    }
}

impl TryFrom<&str> for ProductGroup {
    type Error = CropError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "crops" => Ok(ProductGroup::Crops),
            "fish_meat_eggs" => Ok(ProductGroup::FishMeatEggs),
            "honey_dairy" => Ok(ProductGroup::HoneyDairy),
            "animals" => Ok(ProductGroup::Animals),
            "nuts_dried" => Ok(ProductGroup::NutsDried),
            _ => Err(DomainError::InvalidValue(format!("Invalid product group: {value}")).into()),
        }
    }
}

/// What one of a product is: a kilogram, a tray of 30 eggs, a litre, or one
/// animal. Quantities and prices of the product count in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductUnit {
    Kg,
    Tray30,
    Litre,
    Head,
}

impl ProductUnit {
    pub const ALL: [ProductUnit; 4] = [
        ProductUnit::Kg,
        ProductUnit::Tray30,
        ProductUnit::Litre,
        ProductUnit::Head,
    ];
}

impl From<ProductUnit> for String {
    fn from(value: ProductUnit) -> Self {
        match value {
            ProductUnit::Kg => "kg".to_string(),
            ProductUnit::Tray30 => "tray_30".to_string(),
            ProductUnit::Litre => "litre".to_string(),
            ProductUnit::Head => "head".to_string(),
        }
    }
}

impl TryFrom<&str> for ProductUnit {
    type Error = CropError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "kg" => Ok(ProductUnit::Kg),
            "tray_30" => Ok(ProductUnit::Tray30),
            "litre" => Ok(ProductUnit::Litre),
            "head" => Ok(ProductUnit::Head),
            _ => Err(DomainError::InvalidValue(format!("Invalid product unit: {value}")).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_category_round_trips() {
        for category in CropCategory::ALL {
            let stored = String::from(category);

            assert_eq!(
                CropCategory::try_from(stored.as_str()).expect("category"),
                category,
                "{stored:?} did not round trip"
            );
        }
    }

    #[test]
    fn every_season_round_trips() {
        for season in CropSeason::ALL {
            let stored = String::from(season);

            assert_eq!(
                CropSeason::try_from(stored.as_str()).expect("season"),
                season,
                "{stored:?} did not round trip"
            );
        }
    }

    #[test]
    fn every_group_and_unit_round_trips() {
        for group in ProductGroup::ALL {
            let stored = String::from(group);

            assert_eq!(
                ProductGroup::try_from(stored.as_str()).expect("group"),
                group,
                "{stored:?} did not round trip"
            );
        }

        for unit in ProductUnit::ALL {
            let stored = String::from(unit);

            assert_eq!(
                ProductUnit::try_from(stored.as_str()).expect("unit"),
                unit,
                "{stored:?} did not round trip"
            );
        }
    }

    #[test]
    fn crops_are_the_first_group_on_display() {
        let mut groups = ProductGroup::ALL;
        groups.reverse();
        groups.sort();

        assert_eq!(groups, ProductGroup::ALL);
        assert_eq!(groups[0], ProductGroup::Crops);
    }

    #[test]
    fn an_unknown_stored_value_is_rejected_rather_than_defaulted() {
        assert!(ProductGroup::try_from("fish").is_err());
        assert!(ProductUnit::try_from("tray").is_err());
        assert!(ProductUnit::try_from("KG").is_err());
        assert!(CropCategory::try_from("grain").is_err());
        assert!(CropCategory::try_from("Cereal").is_err());
        assert!(CropSeason::try_from("spring").is_err());
        assert!(CropSeason::try_from("").is_err());
    }
}
