use crate::{
    features::farms::domain::{Crop, FarmError},
    shared::{DomainError, Phone},
};

const MAX_SEARCH_LENGTH: usize = 100;
const MAX_AREA_LENGTH: usize = 60;

/// The word a listing or a report uses for "the farms with no place".
pub const UNKNOWN_AREA: &str = "unknown";

/// One part of a place to keep farms by: the area with that name, or the
/// farms that have no place at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AreaFilter {
    /// Lower case. A governorate is matched without regard to case, so its
    /// name and its slug both find it; zone and sub-zone slugs are lower
    /// case already.
    Named(String),
    Unknown,
}

impl AreaFilter {
    pub fn new(value: String) -> Result<Self, FarmError> {
        let value = value.trim().to_lowercase();

        if value.is_empty() || value.chars().count() > MAX_AREA_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "An area filter must be 1 to {MAX_AREA_LENGTH} characters"
            ))
            .into());
        }

        if value == UNKNOWN_AREA {
            return Ok(Self::Unknown);
        }

        Ok(Self::Named(value))
    }
}

/// A crop to keep farms by. `Empty` is the land a farmer has not painted,
/// not a crop, so it cannot be asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlantedCrop(Crop);

impl PlantedCrop {
    pub fn new(crop: Crop) -> Result<Self, FarmError> {
        if crop == Crop::Empty {
            return Err(DomainError::InvalidValue(
                "`empty` is unpainted land, not a crop to filter by".to_string(),
            )
            .into());
        }

        Ok(Self(crop))
    }

    pub fn crop(&self) -> Crop {
        self.0
    }
}

/// Text to look for inside a farm's name or its owner's phone, whatever the
/// case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FarmSearch(String);

impl FarmSearch {
    pub fn new(value: String) -> Result<Self, FarmError> {
        let value = value.trim().to_string();

        if value.is_empty() || value.chars().count() > MAX_SEARCH_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "A search must be 1 to {MAX_SEARCH_LENGTH} characters"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Which farms a staff listing or a report covers. Every part that is given
/// must hold; a default filter covers every farm.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FarmFilter {
    pub owner: Option<Phone>,
    pub governorate: Option<AreaFilter>,
    pub zone: Option<AreaFilter>,
    pub sub_zone: Option<AreaFilter>,
    /// Farms with at least one cell of this crop.
    pub crop: Option<PlantedCrop>,
    pub search: Option<FarmSearch>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FarmSortKey {
    #[default]
    CreatedAt,
    AreaDunam,
    Name,
}

impl TryFrom<&str> for FarmSortKey {
    type Error = FarmError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "created_at" => Ok(Self::CreatedAt),
            "area_dunam" => Ok(Self::AreaDunam),
            "name" => Ok(Self::Name),
            _ => Err(DomainError::InvalidValue(
                "sort must be one of created_at, area_dunam, name".to_string(),
            )
            .into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    #[default]
    Descending,
}

impl TryFrom<&str> for SortDirection {
    type Error = FarmError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "asc" => Ok(Self::Ascending),
            "desc" => Ok(Self::Descending),
            _ => Err(DomainError::InvalidValue("order must be asc or desc".to_string()).into()),
        }
    }
}

/// The order of a staff listing. The default is newest first.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FarmOrder {
    pub key: FarmSortKey,
    pub direction: SortDirection,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_area_filter_is_lowered_so_a_name_and_a_slug_ask_for_the_same_area() {
        assert_eq!(
            AreaFilter::new("Sulaymaniyah".to_string()).expect("filter"),
            AreaFilter::new(" sulaymaniyah ".to_string()).expect("filter")
        );
    }

    #[test]
    fn the_word_unknown_asks_for_the_farms_with_no_place() {
        assert_eq!(
            AreaFilter::new("unknown".to_string()).expect("filter"),
            AreaFilter::Unknown
        );
        assert_eq!(
            AreaFilter::new("Unknown".to_string()).expect("filter"),
            AreaFilter::Unknown
        );
    }

    #[test]
    fn an_empty_or_over_long_area_filter_is_refused() {
        assert!(AreaFilter::new("  ".to_string()).is_err());
        assert!(AreaFilter::new("a".repeat(MAX_AREA_LENGTH)).is_ok());
        assert!(AreaFilter::new("a".repeat(MAX_AREA_LENGTH + 1)).is_err());
    }

    #[test]
    fn empty_land_is_not_a_crop_to_filter_by() {
        assert!(PlantedCrop::new(Crop::Empty).is_err());
        assert_eq!(
            PlantedCrop::new(Crop::Wheat).expect("crop").crop(),
            Crop::Wheat
        );
    }

    #[test]
    fn a_search_is_trimmed_and_keeps_its_case_and_its_wildcards() {
        assert_eq!(
            FarmSearch::new("  50%_Field ".to_string())
                .expect("search")
                .as_str(),
            "50%_Field",
            "escaping is the repository's business, not the search's"
        );
    }

    #[test]
    fn a_search_length_is_counted_in_characters_not_bytes() {
        assert!(FarmSearch::new("ک".repeat(MAX_SEARCH_LENGTH)).is_ok());
        assert!(FarmSearch::new("ک".repeat(MAX_SEARCH_LENGTH + 1)).is_err());
        assert!(FarmSearch::new("   ".to_string()).is_err());
    }

    #[test]
    fn the_sort_keys_are_the_three_the_dashboard_offers() {
        assert_eq!(
            FarmSortKey::try_from("created_at").expect("key"),
            FarmSortKey::CreatedAt
        );
        assert_eq!(
            FarmSortKey::try_from("area_dunam").expect("key"),
            FarmSortKey::AreaDunam
        );
        assert_eq!(
            FarmSortKey::try_from("name").expect("key"),
            FarmSortKey::Name
        );
        assert!(FarmSortKey::try_from("owner_phone").is_err());
        assert!(FarmSortKey::try_from("Name").is_err());
    }

    #[test]
    fn the_direction_is_asc_or_desc() {
        assert_eq!(
            SortDirection::try_from("asc").expect("direction"),
            SortDirection::Ascending
        );
        assert_eq!(
            SortDirection::try_from("desc").expect("direction"),
            SortDirection::Descending
        );
        assert!(SortDirection::try_from("down").is_err());
    }

    #[test]
    fn the_default_order_is_newest_first() {
        assert_eq!(
            FarmOrder::default(),
            FarmOrder {
                key: FarmSortKey::CreatedAt,
                direction: SortDirection::Descending,
            }
        );
    }
}
