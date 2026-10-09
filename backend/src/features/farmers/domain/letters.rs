use chrono::{DateTime, Utc};
use getset::Getters;

use crate::features::farmers::domain::{LetterLanguage, LetterNumber, LetterPurpose};

/// A support letter the Ministry issued for a farmer. It is a record: once
/// issued it is never changed or removed.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Letter {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    number: LetterNumber,
    farmer_id: i32,
    /// The staff member who issued it.
    staff_id: i32,
    purpose: LetterPurpose,
    language: LetterLanguage,
    created_at: DateTime<Utc>,
}

impl Letter {
    /// The farmer's `sequence`-th letter. The caller must have counted the
    /// farmer's letters where no other letter can be issued in between.
    pub fn issue(
        farmer_id: i32,
        sequence: u64,
        staff_id: i32,
        purpose: LetterPurpose,
        language: LetterLanguage,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id: None,
            number: LetterNumber::issue(now, farmer_id, sequence),
            farmer_id,
            staff_id,
            purpose,
            language,
            created_at: now,
        }
    }

    /// Reconstruct from persisted state.
    pub fn rehydrate(
        id: i32,
        number: LetterNumber,
        farmer_id: i32,
        staff_id: i32,
        purpose: LetterPurpose,
        language: LetterLanguage,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            number,
            farmer_id,
            staff_id,
            purpose,
            language,
            created_at,
        }
    }
}

/// The land under one crop, by the crop's code. The codes belong to the
/// farms feature; a letter only prints them.
#[derive(Clone, Debug, PartialEq)]
pub struct CropHolding {
    pub crop: String,
    pub dunam: f64,
}

/// One farm as a letter lists it. It comes from the farms feature.
#[derive(Clone, Debug, PartialEq)]
pub struct FarmHolding {
    pub id: i32,
    pub name: String,
    /// Where the farm is, by slug, when the farm has a place.
    pub governorate: Option<String>,
    pub zone_slug: Option<String>,
    pub sub_zone_slug: Option<String>,
    pub area_dunam: f64,
    pub crops: Vec<CropHolding>,
}

/// What a letter states about all of a farmer's farms together.
#[derive(Clone, Debug, PartialEq)]
pub struct LetterTotals {
    pub farms: u64,
    pub dunam: f64,
    /// Largest area first, so the main crop leads; ties go by crop code.
    pub crops: Vec<CropHolding>,
}

impl LetterTotals {
    /// Adds the farms up in the order given, so the same farms always give
    /// the same sums to the last digit.
    pub fn of(farms: &[FarmHolding]) -> Self {
        let mut crops: Vec<CropHolding> = Vec::new();

        for holding in farms.iter().flat_map(|farm| &farm.crops) {
            match crops.iter_mut().find(|total| total.crop == holding.crop) {
                Some(total) => total.dunam += holding.dunam,
                None => crops.push(holding.clone()),
            }
        }

        crops.sort_by(|first, second| {
            second
                .dunam
                .total_cmp(&first.dunam)
                .then_with(|| first.crop.cmp(&second.crop))
        });

        Self {
            farms: farms.len() as u64,
            dunam: farms.iter().map(|farm| farm.area_dunam).sum(),
            crops,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn farm(id: i32, area_dunam: f64, crops: &[(&str, f64)]) -> FarmHolding {
        FarmHolding {
            id,
            name: format!("Farm {id}"),
            governorate: None,
            zone_slug: None,
            sub_zone_slug: None,
            area_dunam,
            crops: crops
                .iter()
                .map(|(crop, dunam)| CropHolding {
                    crop: crop.to_string(),
                    dunam: *dunam,
                })
                .collect(),
        }
    }

    #[test]
    fn a_letter_takes_its_number_from_the_time_the_farmer_and_the_count() {
        let now = Utc.with_ymd_and_hms(2026, 10, 9, 8, 0, 0).unwrap();

        let letter = Letter::issue(
            12,
            2,
            3,
            LetterPurpose::new("Bank loan".to_string()).expect("purpose"),
            LetterLanguage::English,
            now,
        );

        assert!(letter.id().is_none());
        assert_eq!(letter.number().as_str(), "JTY-202610-12-2");
        assert_eq!(*letter.created_at(), now);
    }

    #[test]
    fn totals_add_up_farms_area_and_each_crop_across_farms() {
        let totals = LetterTotals::of(&[
            farm(1, 10.0, &[("wheat", 6.0), ("barley", 2.0)]),
            farm(2, 5.5, &[("barley", 5.0)]),
        ]);

        assert_eq!(totals.farms, 2);
        assert_eq!(totals.dunam, 15.5);
        assert_eq!(
            totals.crops,
            vec![
                CropHolding {
                    crop: "barley".to_string(),
                    dunam: 7.0
                },
                CropHolding {
                    crop: "wheat".to_string(),
                    dunam: 6.0
                },
            ],
            "the largest crop over all farms leads"
        );
    }

    #[test]
    fn crops_of_equal_area_are_listed_by_code() {
        let totals = LetterTotals::of(&[farm(1, 4.0, &[("wheat", 2.0), ("barley", 2.0)])]);

        assert_eq!(totals.crops[0].crop, "barley");
    }

    #[test]
    fn a_farmer_without_farms_has_empty_totals_not_guesses() {
        let totals = LetterTotals::of(&[]);

        assert_eq!(totals.farms, 0);
        assert_eq!(totals.dunam, 0.0);
        assert!(totals.crops.is_empty());
    }
}
