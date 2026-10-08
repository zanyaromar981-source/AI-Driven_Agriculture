use chrono::{DateTime, Days, Months, NaiveDate, Utc};
use getset::Getters;

use crate::{
    features::dams::domain::{DamError, DamSlug, PercentFull, ReadingSource},
    shared::DomainError,
};

/// A reading may say the lake holds a little more than the design capacity:
/// a flood can push a reservoir past it, and both numbers are estimates.
const VOLUME_TOLERANCE: f64 = 1.05;

/// How far from "this day last year" a reading may be and still be shown as
/// last year's.
const YEAR_AGO_WINDOW_DAYS: u64 = 45;

/// A reservoir. Dams are reference data: they are seeded by the migration
/// and never created through the API, so there is no `new`.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Dam {
    id: i32,
    slug: DamSlug,
    name_en: String,
    name_ku: String,
    capacity_bn_m3: f64,
}

impl Dam {
    /// Reconstruct from persisted state.
    pub fn rehydrate(
        id: i32,
        slug: DamSlug,
        name_en: String,
        name_ku: String,
        capacity_bn_m3: f64,
    ) -> Self {
        Self {
            id,
            slug,
            name_en,
            name_ku,
            capacity_bn_m3,
        }
    }
}

/// What one data job run measured for one dam on one day.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct DamReading {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    dam_id: i32,
    day: NaiveDate,
    pct_full: PercentFull,
    volume_bn_m3: Option<f64>,
    lake_area_km2: Option<f64>,
    /// Water that can go to farms this season.
    farm_supply_bn_m3: Option<f64>,
    source: ReadingSource,
    updated_at: DateTime<Utc>,
}

impl DamReading {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dam: &Dam,
        day: NaiveDate,
        pct_full: PercentFull,
        volume_bn_m3: Option<f64>,
        lake_area_km2: Option<f64>,
        farm_supply_bn_m3: Option<f64>,
        source: ReadingSource,
    ) -> Result<Self, DamError> {
        not_negative("volume_bn_m3", volume_bn_m3)?;
        not_negative("lake_area_km2", lake_area_km2)?;
        not_negative("farm_supply_bn_m3", farm_supply_bn_m3)?;

        if let Some(volume) = volume_bn_m3
            // The small epsilon keeps a volume of exactly 105% from failing
            // on floating point rounding.
            && volume > dam.capacity_bn_m3 * VOLUME_TOLERANCE + 1e-9
        {
            return Err(DamError::VolumeOverCapacity {
                volume_bn_m3: volume,
                capacity_bn_m3: dam.capacity_bn_m3,
            });
        }

        Ok(Self {
            id: None,
            dam_id: dam.id,
            day,
            pct_full,
            volume_bn_m3,
            lake_area_km2,
            farm_supply_bn_m3,
            source,
            updated_at: Utc::now(),
        })
    }

    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        dam_id: i32,
        day: NaiveDate,
        pct_full: PercentFull,
        volume_bn_m3: Option<f64>,
        lake_area_km2: Option<f64>,
        farm_supply_bn_m3: Option<f64>,
        source: ReadingSource,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            dam_id,
            day,
            pct_full,
            volume_bn_m3,
            lake_area_km2,
            farm_supply_bn_m3,
            source,
            updated_at,
        }
    }
}

fn not_negative(field: &str, value: Option<f64>) -> Result<(), DamError> {
    match value {
        // A NaN is not below zero, so it needs its own check.
        Some(value) if !value.is_finite() || value < 0.0 => {
            Err(DomainError::InvalidValue(format!("{field} must not be negative")).into())
        }
        _ => Ok(()),
    }
}

/// The days in which a reading counts as "a year before" `latest_day`, both
/// ends included. None only at the edge of the calendar.
pub fn year_ago_window(latest_day: NaiveDate) -> Option<(NaiveDate, NaiveDate)> {
    let target = year_before(latest_day)?;

    Some((
        target.checked_sub_days(Days::new(YEAR_AGO_WINDOW_DAYS))?,
        target.checked_add_days(Days::new(YEAR_AGO_WINDOW_DAYS))?,
    ))
}

/// The 29th of February falls back to the 28th.
fn year_before(day: NaiveDate) -> Option<NaiveDate> {
    day.checked_sub_months(Months::new(12))
}

/// A dam as the dashboard shows it: its latest reading and the reading from
/// about a year earlier to compare it with.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct DamStatus {
    dam: Dam,
    latest: Option<DamReading>,
    year_ago: Option<DamReading>,
}

impl DamStatus {
    /// Picks, from `candidates`, the reading closest to one year before the
    /// latest one. Candidates outside the window are ignored, so the caller
    /// may pass more than the window holds. When two are equally close the
    /// earlier one wins.
    pub fn new(dam: Dam, latest: Option<DamReading>, candidates: Vec<DamReading>) -> Self {
        let year_ago = latest.as_ref().and_then(|latest| {
            let target = year_before(latest.day)?;
            let (from, to) = year_ago_window(latest.day)?;

            candidates
                .into_iter()
                .filter(|reading| reading.day >= from && reading.day <= to)
                .min_by_key(|reading| ((reading.day - target).num_days().abs(), reading.day))
        });

        Self {
            dam,
            latest,
            year_ago,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("day")
    }

    /// Darbandikhan: 3.0 billion m3.
    fn dam() -> Dam {
        Dam::rehydrate(
            2,
            DamSlug::new("darbandikhan".to_string()).expect("slug"),
            "Darbandikhan".to_string(),
            "دەربەندیخان".to_string(),
            3.0,
        )
    }

    fn reading_with(
        volume: Option<f64>,
        lake: Option<f64>,
        supply: Option<f64>,
    ) -> Result<DamReading, DamError> {
        DamReading::new(
            &dam(),
            day(2026, 10, 1),
            PercentFull::new(50.0).expect("percent"),
            volume,
            lake,
            supply,
            ReadingSource::new("test".to_string()).expect("source"),
        )
    }

    fn reading_on(on: NaiveDate) -> DamReading {
        DamReading::new(
            &dam(),
            on,
            PercentFull::new(50.0).expect("percent"),
            None,
            None,
            None,
            ReadingSource::new("test".to_string()).expect("source"),
        )
        .expect("reading")
    }

    #[test]
    fn a_new_reading_belongs_to_its_dam_and_is_not_yet_persisted() {
        let reading = reading_with(Some(1.5), Some(80.0), Some(0.6)).expect("reading");

        assert_eq!(*reading.id(), None);
        assert_eq!(*reading.dam_id(), 2);
        assert_eq!(*reading.volume_bn_m3(), Some(1.5));
    }

    #[test]
    fn the_optional_measurements_may_all_be_missing() {
        assert!(reading_with(None, None, None).is_ok());
    }

    #[test]
    fn a_volume_up_to_five_percent_over_capacity_is_accepted() {
        assert!(reading_with(Some(3.0), None, None).is_ok());
        assert!(
            reading_with(Some(3.15), None, None).is_ok(),
            "exactly 105% is still inside the tolerance"
        );
    }

    #[test]
    fn a_volume_more_than_five_percent_over_capacity_is_refused() {
        let result = reading_with(Some(3.16), None, None);

        assert!(matches!(
            result,
            Err(DamError::VolumeOverCapacity {
                volume_bn_m3,
                capacity_bn_m3,
            }) if volume_bn_m3 == 3.16 && capacity_bn_m3 == 3.0
        ));
    }

    #[test]
    fn a_negative_measurement_is_refused() {
        assert!(reading_with(Some(-0.1), None, None).is_err());
        assert!(reading_with(None, Some(-1.0), None).is_err());
        assert!(reading_with(None, None, Some(-0.5)).is_err());
    }

    #[test]
    fn a_measurement_that_is_not_a_number_is_refused() {
        assert!(reading_with(Some(f64::NAN), None, None).is_err());
        assert!(reading_with(None, Some(f64::INFINITY), None).is_err());
    }

    #[test]
    fn zero_is_a_valid_measurement() {
        assert!(reading_with(Some(0.0), Some(0.0), Some(0.0)).is_ok());
    }

    #[test]
    fn the_window_is_forty_five_days_either_side_of_a_year_before() {
        assert_eq!(
            year_ago_window(day(2026, 10, 1)),
            Some((day(2025, 8, 17), day(2025, 11, 15)))
        );
    }

    #[test]
    fn a_year_before_the_leap_day_is_the_end_of_february() {
        assert_eq!(year_before(day(2024, 2, 29)), Some(day(2023, 2, 28)));
    }

    #[test]
    fn a_dam_without_readings_has_no_latest_and_no_year_ago() {
        let status = DamStatus::new(dam(), None, vec![reading_on(day(2025, 10, 1))]);

        assert!(status.latest().is_none());
        assert!(
            status.year_ago().is_none(),
            "there is nothing to compare a missing reading with"
        );
    }

    #[test]
    fn the_reading_closest_to_a_year_before_is_chosen() {
        let status = DamStatus::new(
            dam(),
            Some(reading_on(day(2026, 10, 1))),
            vec![
                reading_on(day(2025, 9, 1)),
                reading_on(day(2025, 10, 4)),
                reading_on(day(2025, 11, 10)),
            ],
        );

        assert_eq!(
            status.year_ago().as_ref().map(|reading| *reading.day()),
            Some(day(2025, 10, 4))
        );
    }

    #[test]
    fn the_edges_of_the_window_count_and_the_day_after_does_not() {
        let latest = || Some(reading_on(day(2026, 10, 1)));

        let on_the_edge = DamStatus::new(dam(), latest(), vec![reading_on(day(2025, 11, 15))]);
        let past_the_edge = DamStatus::new(dam(), latest(), vec![reading_on(day(2025, 11, 16))]);
        let before_the_edge = DamStatus::new(dam(), latest(), vec![reading_on(day(2025, 8, 16))]);

        assert!(on_the_edge.year_ago().is_some());
        assert!(
            past_the_edge.year_ago().is_none(),
            "46 days away is not last year's reading"
        );
        assert!(before_the_edge.year_ago().is_none());
    }

    #[test]
    fn the_earlier_reading_wins_when_two_are_equally_close() {
        let status = DamStatus::new(
            dam(),
            Some(reading_on(day(2026, 10, 1))),
            vec![reading_on(day(2025, 10, 3)), reading_on(day(2025, 9, 29))],
        );

        assert_eq!(
            status.year_ago().as_ref().map(|reading| *reading.day()),
            Some(day(2025, 9, 29))
        );
    }

    #[test]
    fn the_latest_reading_is_never_its_own_year_ago() {
        let latest = reading_on(day(2026, 10, 1));
        let status = DamStatus::new(dam(), Some(latest.clone()), vec![latest]);

        assert!(status.year_ago().is_none());
    }
}
