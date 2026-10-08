use chrono::{DateTime, Utc};
use getset::Getters;

use crate::features::zones::domain::{
    Crop, Dryness, GreennessPctVsNormal, Month, RainPctOfNormal, ReadingSource, WaterNeed,
    ZoneError, ZoneSlug,
};

/// One of the districts the Kurdistan Region is shown as. Zones are reference
/// data seeded by a migration: the app reads them and never creates one.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct Zone {
    id: i32,
    slug: ZoneSlug,
    name_en: String,
    /// The Sorani name.
    name_ku: String,
    governorate: String,
}

impl Zone {
    /// Reconstruct from persisted state.
    pub fn rehydrate(
        id: i32,
        slug: ZoneSlug,
        name_en: String,
        name_ku: String,
        governorate: String,
    ) -> Self {
        Self {
            id,
            slug,
            name_en,
            name_ku,
            governorate,
        }
    }
}

/// A part of a zone. Its slug is unique inside its zone only.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct SubZone {
    id: i32,
    zone_id: i32,
    slug: ZoneSlug,
    name_en: String,
    name_ku: String,
}

impl SubZone {
    /// Reconstruct from persisted state.
    pub fn rehydrate(
        id: i32,
        zone_id: i32,
        slug: ZoneSlug,
        name_en: String,
        name_ku: String,
    ) -> Self {
        Self {
            id,
            zone_id,
            slug,
            name_en,
            name_ku,
        }
    }
}

/// What the data jobs measured for one zone in one month. There is at most
/// one per zone and month: a later push replaces the earlier one.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct ZoneReading {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    zone_id: i32,
    month: Month,
    dryness: Dryness,
    rain_pct_of_normal: Option<RainPctOfNormal>,
    greenness_pct_vs_normal: Option<GreennessPctVsNormal>,
    water_need: Option<WaterNeed>,
    /// True when farmers should hold back nitrogen this month.
    nitrogen_hold: bool,
    /// Best first. May be empty.
    best_crops: Vec<Crop>,
    source: ReadingSource,
    updated_at: DateTime<Utc>,
}

impl ZoneReading {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        zone_id: i32,
        month: Month,
        dryness: Dryness,
        rain_pct_of_normal: Option<RainPctOfNormal>,
        greenness_pct_vs_normal: Option<GreennessPctVsNormal>,
        water_need: Option<WaterNeed>,
        nitrogen_hold: bool,
        best_crops: Vec<Crop>,
        source: ReadingSource,
    ) -> Result<Self, ZoneError> {
        // The list is a ranking, and a crop cannot hold two places in it.
        for (index, crop) in best_crops.iter().enumerate() {
            if best_crops[..index].contains(crop) {
                return Err(ZoneError::RepeatedCrop(String::from(*crop)));
            }
        }

        Ok(Self {
            id: None,
            zone_id,
            month,
            dryness,
            rain_pct_of_normal,
            greenness_pct_vs_normal,
            water_need,
            nitrogen_hold,
            best_crops,
            source,
            updated_at: Utc::now(),
        })
    }

    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        zone_id: i32,
        month: Month,
        dryness: Dryness,
        rain_pct_of_normal: Option<RainPctOfNormal>,
        greenness_pct_vs_normal: Option<GreennessPctVsNormal>,
        water_need: Option<WaterNeed>,
        nitrogen_hold: bool,
        best_crops: Vec<Crop>,
        source: ReadingSource,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            zone_id,
            month,
            dryness,
            rain_pct_of_normal,
            greenness_pct_vs_normal,
            water_need,
            nitrogen_hold,
            best_crops,
            source,
            updated_at,
        }
    }
}

/// The dryness of one sub-zone in one month. At most one per sub-zone and
/// month.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct SubZoneReading {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    sub_zone_id: i32,
    month: Month,
    dryness: Dryness,
    updated_at: DateTime<Utc>,
}

impl SubZoneReading {
    /// Every part is already a validated value object, so nothing can fail.
    pub fn new(sub_zone_id: i32, month: Month, dryness: Dryness) -> Self {
        Self {
            id: None,
            sub_zone_id,
            month,
            dryness,
            updated_at: Utc::now(),
        }
    }

    /// Reconstruct from persisted state.
    pub fn rehydrate(
        id: i32,
        sub_zone_id: i32,
        month: Month,
        dryness: Dryness,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            sub_zone_id,
            month,
            dryness,
            updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(best_crops: Vec<Crop>) -> Result<ZoneReading, ZoneError> {
        ZoneReading::new(
            1,
            Month::parse("2026-03").expect("month"),
            Dryness::new(70).expect("dryness"),
            None,
            None,
            None,
            false,
            best_crops,
            ReadingSource::new("chirps+modis".to_string()).expect("source"),
        )
    }

    #[test]
    fn a_new_reading_is_not_yet_persisted() {
        let reading = reading(vec![]).expect("reading");

        assert_eq!(*reading.id(), None);
        assert_eq!(*reading.zone_id(), 1);
    }

    #[test]
    fn a_reading_can_carry_no_crop_advice() {
        assert!(reading(vec![]).expect("reading").best_crops().is_empty());
    }

    #[test]
    fn the_best_crops_keep_the_order_they_were_given_in() {
        let reading = reading(vec![Crop::Barley, Crop::Wheat, Crop::Chickpea]).expect("reading");

        assert_eq!(
            reading.best_crops(),
            &vec![Crop::Barley, Crop::Wheat, Crop::Chickpea],
            "the order is the ranking, so it must not be sorted"
        );
    }

    #[test]
    fn a_crop_cannot_be_ranked_twice() {
        assert!(matches!(
            reading(vec![Crop::Wheat, Crop::Barley, Crop::Wheat]),
            Err(ZoneError::RepeatedCrop(code)) if code == "wheat"
        ));
    }

    #[test]
    fn a_new_sub_zone_reading_is_not_yet_persisted() {
        let reading = SubZoneReading::new(
            4,
            Month::parse("2026-03").expect("month"),
            Dryness::new(55).expect("dryness"),
        );

        assert_eq!(*reading.id(), None);
        assert_eq!(*reading.sub_zone_id(), 4);
    }
}
