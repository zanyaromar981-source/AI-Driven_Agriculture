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
    fn an_unknown_stored_value_is_rejected_rather_than_defaulted() {
        assert!(CropCategory::try_from("grain").is_err());
        assert!(CropCategory::try_from("Cereal").is_err());
        assert!(CropSeason::try_from("spring").is_err());
        assert!(CropSeason::try_from("").is_err());
    }
}
