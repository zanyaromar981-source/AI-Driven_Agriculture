use chrono::{DateTime, Utc};
use getset::Getters;

use crate::features::crops::domain::{
    CropCategory, CropCode, CropColor, CropName, CropSeason, SortOrder, YieldKgPerDunam,
};

/// Everything about a crop that staff may change. The code is not here: it
/// names the crop and never changes.
#[derive(Clone, Debug, PartialEq)]
pub struct CropDetails {
    pub name_en: CropName,
    /// None until the Sorani name has been written.
    pub name_ku: Option<CropName>,
    pub color: CropColor,
    pub category: CropCategory,
    pub season: CropSeason,
    /// None = not known.
    pub yield_kg_per_dunam: Option<YieldKgPerDunam>,
    /// A crop switched off is no longer offered for new farms, listings and
    /// prices; what was stored with it stays.
    pub active: bool,
    pub sort_order: SortOrder,
}

/// One crop of the list farms, listings and prices choose from. Its code is
/// its identity, so there is no numeric id.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Crop {
    code: CropCode,
    details: CropDetails,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Crop {
    /// A crop staff are adding at `now`. Its parts were validated when they
    /// were built, so nothing is left that could fail here.
    pub fn new(code: CropCode, details: CropDetails, now: DateTime<Utc>) -> Self {
        Self {
            code,
            details,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstruct from persisted state.
    pub fn rehydrate(
        code: CropCode,
        details: CropDetails,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            code,
            details,
            created_at,
            updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn a_new_crop_was_created_and_last_changed_at_the_same_moment() {
        let now = Utc.with_ymd_and_hms(2026, 10, 9, 12, 0, 0).unwrap();

        let crop = Crop::new(
            CropCode::new("rice".to_string()).expect("code"),
            CropDetails {
                name_en: CropName::new("Rice".to_string()).expect("name"),
                name_ku: None,
                color: CropColor::new("#aabbcc".to_string()).expect("colour"),
                category: CropCategory::Cereal,
                season: CropSeason::Summer,
                yield_kg_per_dunam: None,
                active: true,
                sort_order: SortOrder::new(170).expect("order"),
            },
            now,
        );

        assert_eq!(crop.code().as_str(), "rice");
        assert_eq!(*crop.created_at(), now);
        assert_eq!(*crop.updated_at(), now);
        assert!(
            crop.details().yield_kg_per_dunam.is_none(),
            "a yield nobody entered stays unknown"
        );
    }
}
