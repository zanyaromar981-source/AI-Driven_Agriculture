//! What the dashboard reads: the region for one month, one zone in depth,
//! and one calendar month compared across years. Each view is built in
//! memory from zones and the readings loaded for it.

use std::{
    cmp::Reverse,
    collections::{BTreeMap, HashMap},
};

use getset::Getters;

use crate::features::zones::domain::{
    Dryness, Month, SubZone, SubZoneReading, YearComparison, Zone, ZoneReading, ZoneSlug,
    statistics::{mean, ranks, round_to_one_decimal},
};

/// How many of the driest zones the summary names.
const DRIEST_SHOWN: usize = 5;

/// How many earlier years a zone's history goes back.
const HISTORY_YEARS: usize = 5;

/// One zone on the region map for one month. A zone the jobs have not
/// measured that month is still listed, with nothing measured.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct ZoneOverview {
    zone: Zone,
    reading: Option<ZoneReading>,
    /// Dryness minus the same month one year earlier.
    change_vs_last_year: Option<i32>,
    /// 1 = driest.
    rank: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct RegionSummary {
    zones_with_data: usize,
    /// Over the zones that have data.
    average_dryness: Option<f64>,
    /// Over the zones that have data in both years, so a zone measured in
    /// only one of them cannot fake a change.
    change_vs_last_year: Option<f64>,
    /// Driest first.
    driest: Vec<ZoneSlug>,
    nitrogen_hold: Vec<ZoneSlug>,
}

#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct RegionOverview {
    month: Month,
    zones: Vec<ZoneOverview>,
    summary: RegionSummary,
}

impl RegionOverview {
    /// `readings` are those of `month`, `last_year` those of the same month
    /// one year earlier. Zones keep the order they are given in.
    pub fn build(
        month: Month,
        zones: Vec<Zone>,
        readings: &[ZoneReading],
        last_year: &[ZoneReading],
    ) -> Self {
        let now: HashMap<i32, &ZoneReading> = readings
            .iter()
            .filter(|reading| *reading.month() == month)
            .map(|reading| (*reading.zone_id(), reading))
            .collect();

        let then: HashMap<i32, Dryness> = last_year
            .iter()
            .filter(|reading| Some(*reading.month()) == month.a_year_earlier())
            .map(|reading| (*reading.zone_id(), *reading.dryness()))
            .collect();

        // Only zones that still exist are ranked and averaged.
        let measured: Vec<(&Zone, &ZoneReading)> = zones
            .iter()
            .filter_map(|zone| now.get(zone.id()).map(|reading| (zone, *reading)))
            .collect();

        let ranks = ranks(
            &measured
                .iter()
                .map(|(zone, reading)| (*zone.id(), *reading.dryness()))
                .collect::<Vec<_>>(),
        );

        let summary = Self::summarise(&measured, &then);

        let zones = zones
            .into_iter()
            .map(|zone| {
                let reading = now.get(zone.id()).map(|reading| (*reading).clone());

                let change_vs_last_year = reading.as_ref().and_then(|reading| {
                    then.get(zone.id())
                        .map(|earlier| reading.dryness().change_from(*earlier))
                });

                ZoneOverview {
                    rank: ranks.get(zone.id()).copied(),
                    change_vs_last_year,
                    reading,
                    zone,
                }
            })
            .collect();

        Self {
            month,
            zones,
            summary,
        }
    }

    fn summarise(
        measured: &[(&Zone, &ZoneReading)],
        then: &HashMap<i32, Dryness>,
    ) -> RegionSummary {
        let all: Vec<Dryness> = measured
            .iter()
            .map(|(_, reading)| *reading.dryness())
            .collect();

        let (both_now, both_then): (Vec<Dryness>, Vec<Dryness>) = measured
            .iter()
            .filter_map(|(zone, reading)| {
                then.get(zone.id())
                    .map(|earlier| (*reading.dryness(), *earlier))
            })
            .unzip();

        let change_vs_last_year = match (mean(&both_now), mean(&both_then)) {
            (Some(now), Some(then)) => Some(round_to_one_decimal(now - then)),
            _ => None,
        };

        // Equally dry zones are named in slug order, so the list is stable
        // from one request to the next.
        let mut driest_first = measured.to_vec();
        driest_first.sort_by(|(first_zone, first), (second_zone, second)| {
            second
                .dryness()
                .cmp(first.dryness())
                .then_with(|| first_zone.slug().cmp(second_zone.slug()))
        });

        RegionSummary {
            zones_with_data: measured.len(),
            average_dryness: mean(&all).map(round_to_one_decimal),
            change_vs_last_year,
            driest: driest_first
                .iter()
                .take(DRIEST_SHOWN)
                .map(|(zone, _)| zone.slug().clone())
                .collect(),
            nitrogen_hold: measured
                .iter()
                .filter(|(_, reading)| *reading.nitrogen_hold())
                .map(|(zone, _)| zone.slug().clone())
                .collect(),
        }
    }
}

/// A zone's reading with its place among the zones measured that month.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct RankedReading {
    reading: ZoneReading,
    /// 1 = driest.
    rank: u32,
    /// How many zones were ranked that month.
    rank_of: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct SubZoneDryness {
    sub_zone: SubZone,
    dryness: Option<Dryness>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct YearDryness {
    year: i32,
    dryness: Dryness,
}

#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct ZoneDetail {
    zone: Zone,
    month: Month,
    reading: Option<RankedReading>,
    /// Driest first, the ones not measured last.
    sub_zones: Vec<SubZoneDryness>,
    /// The same calendar month in earlier years that have data, oldest first.
    history: Vec<YearDryness>,
}

impl ZoneDetail {
    /// `month_readings` are every zone's readings for `month` (the zone is
    /// ranked among them). `zone_readings` are this zone's readings for any
    /// month (its history is picked out of them).
    pub fn build(
        zone: Zone,
        month: Month,
        month_readings: &[ZoneReading],
        zone_readings: &[ZoneReading],
        sub_zones: Vec<SubZone>,
        sub_zone_readings: &[SubZoneReading],
    ) -> Self {
        let month_readings: Vec<&ZoneReading> = month_readings
            .iter()
            .filter(|reading| *reading.month() == month)
            .collect();

        let ranks = ranks(
            &month_readings
                .iter()
                .map(|reading| (*reading.zone_id(), *reading.dryness()))
                .collect::<Vec<_>>(),
        );

        let reading = month_readings
            .iter()
            .find(|reading| reading.zone_id() == zone.id())
            .and_then(|reading| {
                ranks.get(zone.id()).map(|rank| RankedReading {
                    reading: (*reading).clone(),
                    rank: *rank,
                    rank_of: month_readings.len(),
                })
            });

        let mut history: Vec<YearDryness> = zone_readings
            .iter()
            .filter(|reading| reading.zone_id() == zone.id())
            .filter(|reading| {
                reading.month().number() == month.number() && reading.month().year() < month.year()
            })
            .map(|reading| YearDryness {
                year: reading.month().year(),
                dryness: *reading.dryness(),
            })
            .collect();

        // The nearest years are the ones kept, then shown oldest first.
        history.sort_by_key(|point| point.year);
        let history = history.split_off(history.len().saturating_sub(HISTORY_YEARS));

        let dryness_by_sub_zone: HashMap<i32, Dryness> = sub_zone_readings
            .iter()
            .filter(|reading| *reading.month() == month)
            .map(|reading| (*reading.sub_zone_id(), *reading.dryness()))
            .collect();

        let mut sub_zones: Vec<SubZoneDryness> = sub_zones
            .into_iter()
            .filter(|sub_zone| sub_zone.zone_id() == zone.id())
            .map(|sub_zone| SubZoneDryness {
                dryness: dryness_by_sub_zone.get(sub_zone.id()).copied(),
                sub_zone,
            })
            .collect();

        // A stable sort: equally dry and unmeasured sub-zones keep the order
        // they were given in. `None` sorts below every `Some`, so reversing
        // the order puts it last.
        sub_zones.sort_by_key(|row| Reverse(row.dryness));

        Self {
            zone,
            month,
            reading,
            sub_zones,
            history,
        }
    }
}

/// One zone in the two years being compared.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct ZoneComparison {
    zone: Zone,
    dryness: Option<Dryness>,
    dryness_with: Option<Dryness>,
    /// `dryness` minus `dryness_with`, when both are there.
    change: Option<i32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct YearAverage {
    year: i32,
    average_dryness: f64,
}

#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct RegionComparison {
    comparison: YearComparison,
    /// Largest change first, in either direction; zones missing a year last.
    zones: Vec<ZoneComparison>,
    /// The region's average for that calendar month in every year that has
    /// data, oldest first.
    region: Vec<YearAverage>,
}

impl RegionComparison {
    /// `readings` are those of the compared calendar month in every year.
    pub fn build(comparison: YearComparison, zones: Vec<Zone>, readings: &[ZoneReading]) -> Self {
        let known: HashMap<i32, &Zone> = zones.iter().map(|zone| (*zone.id(), zone)).collect();

        let readings: Vec<&ZoneReading> = readings
            .iter()
            .filter(|reading| reading.month().number() == comparison.calendar_month())
            .filter(|reading| known.contains_key(reading.zone_id()))
            .collect();

        let dryness_in = |month: Month| -> HashMap<i32, Dryness> {
            readings
                .iter()
                .filter(|reading| *reading.month() == month)
                .map(|reading| (*reading.zone_id(), *reading.dryness()))
                .collect()
        };

        let now = dryness_in(comparison.month());
        let with = dryness_in(comparison.with_month());

        let mut per_year: BTreeMap<i32, Vec<Dryness>> = BTreeMap::new();

        for reading in &readings {
            per_year
                .entry(reading.month().year())
                .or_default()
                .push(*reading.dryness());
        }

        let region = per_year
            .into_iter()
            .filter_map(|(year, values)| {
                mean(&values).map(|average| YearAverage {
                    year,
                    average_dryness: round_to_one_decimal(average),
                })
            })
            .collect();

        let mut zones: Vec<ZoneComparison> = zones
            .into_iter()
            .map(|zone| {
                let dryness = now.get(zone.id()).copied();
                let dryness_with = with.get(zone.id()).copied();

                ZoneComparison {
                    change: match (dryness, dryness_with) {
                        (Some(dryness), Some(earlier)) => Some(dryness.change_from(earlier)),
                        _ => None,
                    },
                    dryness,
                    dryness_with,
                    zone,
                }
            })
            .collect();

        // A stable sort: equal changes and zones missing a year keep the
        // order they were given in. `None` sorts below every `Some`.
        zones.sort_by_key(|row| Reverse(row.change.map(i32::abs)));

        Self {
            comparison,
            zones,
            region,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::zones::domain::ReadingSource;

    fn month(value: &str) -> Month {
        Month::parse(value).expect("month")
    }

    fn zone(id: i32, slug: &str) -> Zone {
        Zone::rehydrate(
            id,
            ZoneSlug::new(slug.to_string()).expect("slug"),
            slug.to_string(),
            slug.to_string(),
            "Sulaymaniyah".to_string(),
        )
    }

    fn zones() -> Vec<Zone> {
        vec![
            zone(1, "chamchamal"),
            zone(2, "kalar"),
            zone(3, "kifri"),
            zone(4, "penjwen"),
        ]
    }

    fn reading(zone_id: i32, at: &str, dryness: i32) -> ZoneReading {
        holding(zone_id, at, dryness, false)
    }

    fn holding(zone_id: i32, at: &str, dryness: i32, nitrogen_hold: bool) -> ZoneReading {
        ZoneReading::new(
            zone_id,
            month(at),
            Dryness::new(dryness).expect("dryness"),
            None,
            None,
            None,
            nitrogen_hold,
            vec![],
            ReadingSource::new("test".to_string()).expect("source"),
        )
        .expect("reading")
    }

    fn sub_zone(id: i32, slug: &str) -> SubZone {
        SubZone::rehydrate(
            id,
            1,
            ZoneSlug::new(slug.to_string()).expect("slug"),
            slug.to_string(),
            slug.to_string(),
        )
    }

    fn sub_reading(sub_zone_id: i32, at: &str, dryness: i32) -> SubZoneReading {
        SubZoneReading::new(
            sub_zone_id,
            month(at),
            Dryness::new(dryness).expect("dryness"),
        )
    }

    fn slugs(list: &[ZoneSlug]) -> Vec<&str> {
        list.iter().map(ZoneSlug::as_str).collect()
    }

    #[test]
    fn every_zone_is_listed_even_without_a_reading() {
        let overview =
            RegionOverview::build(month("2026-03"), zones(), &[reading(2, "2026-03", 70)], &[]);

        assert_eq!(overview.zones().len(), 4);

        let unmeasured = &overview.zones()[0];

        assert!(unmeasured.reading().is_none());
        assert_eq!(*unmeasured.rank(), None, "no reading, no rank");
        assert_eq!(*unmeasured.change_vs_last_year(), None);
    }

    #[test]
    fn zones_keep_the_order_they_were_given_in() {
        let overview = RegionOverview::build(month("2026-03"), zones(), &[], &[]);

        assert_eq!(
            overview
                .zones()
                .iter()
                .map(|row| row.zone().slug().as_str())
                .collect::<Vec<_>>(),
            vec!["chamchamal", "kalar", "kifri", "penjwen"]
        );
    }

    #[test]
    fn the_driest_zone_is_rank_one_on_the_overview() {
        let overview = RegionOverview::build(
            month("2026-03"),
            zones(),
            &[
                reading(1, "2026-03", 40),
                reading(2, "2026-03", 90),
                reading(3, "2026-03", 65),
            ],
            &[],
        );

        let rank_of = |index: usize| *overview.zones()[index].rank();

        assert_eq!(rank_of(0), Some(3));
        assert_eq!(rank_of(1), Some(1));
        assert_eq!(rank_of(2), Some(2));
        assert_eq!(rank_of(3), None);
    }

    #[test]
    fn the_change_is_this_year_minus_last_year() {
        let overview = RegionOverview::build(
            month("2026-03"),
            zones(),
            &[reading(1, "2026-03", 70), reading(2, "2026-03", 30)],
            &[reading(1, "2025-03", 55)],
        );

        assert_eq!(*overview.zones()[0].change_vs_last_year(), Some(15));
        assert_eq!(
            *overview.zones()[1].change_vs_last_year(),
            None,
            "no reading a year earlier, so no change rather than a change from zero"
        );
    }

    #[test]
    fn a_reading_from_another_month_is_never_mistaken_for_last_year() {
        let overview = RegionOverview::build(
            month("2026-03"),
            zones(),
            &[reading(1, "2026-03", 70)],
            &[reading(1, "2025-04", 10), reading(1, "2024-03", 10)],
        );

        assert_eq!(*overview.zones()[0].change_vs_last_year(), None);
    }

    #[test]
    fn the_average_is_over_the_zones_that_have_data() {
        let overview = RegionOverview::build(
            month("2026-03"),
            zones(),
            &[
                reading(1, "2026-03", 10),
                reading(2, "2026-03", 20),
                reading(3, "2026-03", 25),
            ],
            &[],
        );

        assert_eq!(*overview.summary().zones_with_data(), 3);
        assert_eq!(
            *overview.summary().average_dryness(),
            Some(18.3),
            "55 over the 3 measured zones, not over all 4"
        );
    }

    #[test]
    fn the_summary_change_only_counts_zones_measured_in_both_years() {
        let overview = RegionOverview::build(
            month("2026-03"),
            zones(),
            &[
                reading(1, "2026-03", 70),
                reading(2, "2026-03", 60),
                // Measured this year only: must not pull the change up.
                reading(3, "2026-03", 100),
            ],
            &[
                reading(1, "2025-03", 50),
                reading(2, "2025-03", 55),
                // Measured last year only: must not pull the change down.
                reading(4, "2025-03", 0),
            ],
        );

        assert_eq!(
            *overview.summary().change_vs_last_year(),
            Some(12.5),
            "(70 + 60) / 2 minus (50 + 55) / 2"
        );
    }

    #[test]
    fn a_region_with_no_data_has_no_average_and_no_change() {
        let overview = RegionOverview::build(month("2026-03"), zones(), &[], &[]);

        assert_eq!(*overview.summary().zones_with_data(), 0);
        assert_eq!(*overview.summary().average_dryness(), None);
        assert_eq!(*overview.summary().change_vs_last_year(), None);
        assert!(overview.summary().driest().is_empty());
        assert!(overview.summary().nitrogen_hold().is_empty());
    }

    #[test]
    fn the_summary_names_at_most_five_driest_zones_driest_first() {
        let many: Vec<Zone> = (1..=7).map(|id| zone(id, &format!("zone-{id}"))).collect();
        let readings: Vec<ZoneReading> =
            (1..=7).map(|id| reading(id, "2026-03", id * 10)).collect();

        let overview = RegionOverview::build(month("2026-03"), many, &readings, &[]);

        assert_eq!(
            slugs(overview.summary().driest()),
            vec!["zone-7", "zone-6", "zone-5", "zone-4", "zone-3"]
        );
    }

    #[test]
    fn equally_dry_zones_are_named_in_slug_order() {
        let overview = RegionOverview::build(
            month("2026-03"),
            zones(),
            &[reading(3, "2026-03", 80), reading(2, "2026-03", 80)],
            &[],
        );

        assert_eq!(slugs(overview.summary().driest()), vec!["kalar", "kifri"]);
    }

    #[test]
    fn the_summary_lists_the_zones_told_to_hold_nitrogen() {
        let overview = RegionOverview::build(
            month("2026-03"),
            zones(),
            &[
                holding(1, "2026-03", 70, true),
                holding(2, "2026-03", 30, false),
                holding(4, "2026-03", 85, true),
            ],
            &[],
        );

        assert_eq!(
            slugs(overview.summary().nitrogen_hold()),
            vec!["chamchamal", "penjwen"]
        );
    }

    #[test]
    fn a_zone_is_ranked_among_the_zones_measured_that_month() {
        let detail = ZoneDetail::build(
            zone(1, "chamchamal"),
            month("2026-03"),
            &[
                reading(1, "2026-03", 60),
                reading(2, "2026-03", 90),
                reading(3, "2026-03", 20),
            ],
            &[],
            vec![],
            &[],
        );

        let ranked = detail.reading().as_ref().expect("reading");

        assert_eq!(*ranked.rank(), 2);
        assert_eq!(*ranked.rank_of(), 3);
        assert_eq!(ranked.reading().dryness().value(), 60);
    }

    #[test]
    fn a_zone_without_a_reading_that_month_has_none_and_no_rank() {
        let detail = ZoneDetail::build(
            zone(1, "chamchamal"),
            month("2026-03"),
            &[reading(2, "2026-03", 90)],
            &[reading(1, "2026-02", 50)],
            vec![],
            &[],
        );

        assert!(detail.reading().is_none());
    }

    #[test]
    fn the_history_is_the_same_calendar_month_in_earlier_years_oldest_first() {
        let detail = ZoneDetail::build(
            zone(1, "chamchamal"),
            month("2026-03"),
            &[],
            &[
                reading(1, "2025-03", 55),
                reading(1, "2023-03", 40),
                // Another month, the month itself and a later year do not belong.
                reading(1, "2025-04", 99),
                reading(1, "2026-03", 70),
                reading(1, "2027-03", 99),
            ],
            vec![],
            &[],
        );

        assert_eq!(
            detail
                .history()
                .iter()
                .map(|point| (*point.year(), point.dryness().value()))
                .collect::<Vec<_>>(),
            vec![(2023, 40), (2025, 55)],
            "a year without data is left out, not filled in"
        );
    }

    #[test]
    fn the_history_keeps_the_five_nearest_years() {
        let readings: Vec<ZoneReading> = (2018..=2025)
            .map(|year| reading(1, &format!("{year}-03"), year - 2000))
            .collect();

        let detail = ZoneDetail::build(
            zone(1, "chamchamal"),
            month("2026-03"),
            &[],
            &readings,
            vec![],
            &[],
        );

        assert_eq!(
            detail
                .history()
                .iter()
                .map(|point| *point.year())
                .collect::<Vec<_>>(),
            vec![2021, 2022, 2023, 2024, 2025]
        );
    }

    #[test]
    fn sub_zones_are_listed_driest_first_with_the_unmeasured_last() {
        let detail = ZoneDetail::build(
            zone(1, "chamchamal"),
            month("2026-03"),
            &[],
            &[],
            vec![
                sub_zone(1, "markaz-chamchamal"),
                sub_zone(2, "aghjalar"),
                sub_zone(3, "sangaw"),
                sub_zone(4, "qadir-karam"),
            ],
            &[
                sub_reading(2, "2026-03", 45),
                sub_reading(3, "2026-03", 88),
                // Another month does not count.
                sub_reading(1, "2026-02", 99),
            ],
        );

        assert_eq!(
            detail
                .sub_zones()
                .iter()
                .map(|row| (
                    row.sub_zone().slug().as_str(),
                    row.dryness().map(|dryness| dryness.value())
                ))
                .collect::<Vec<_>>(),
            vec![
                ("sangaw", Some(88)),
                ("aghjalar", Some(45)),
                ("markaz-chamchamal", None),
                ("qadir-karam", None),
            ]
        );
    }

    fn comparison() -> YearComparison {
        YearComparison::new(2026, 2025, 3).expect("comparison")
    }

    #[test]
    fn the_largest_change_in_either_direction_comes_first() {
        let compared = RegionComparison::build(
            comparison(),
            zones(),
            &[
                reading(1, "2026-03", 60),
                reading(1, "2025-03", 55),
                reading(2, "2026-03", 30),
                reading(2, "2025-03", 70),
                reading(3, "2026-03", 80),
                reading(3, "2025-03", 60),
            ],
        );

        assert_eq!(
            compared
                .zones()
                .iter()
                .map(|row| (row.zone().slug().as_str(), *row.change()))
                .collect::<Vec<_>>(),
            vec![
                ("kalar", Some(-40)),
                ("kifri", Some(20)),
                ("chamchamal", Some(5)),
                ("penjwen", None),
            ]
        );
    }

    #[test]
    fn a_zone_missing_either_year_goes_last_and_keeps_the_value_it_has() {
        let compared = RegionComparison::build(
            comparison(),
            zones(),
            &[
                reading(1, "2026-03", 60),
                reading(2, "2025-03", 70),
                reading(3, "2026-03", 50),
                reading(3, "2025-03", 50),
            ],
        );

        let rows: Vec<_> = compared
            .zones()
            .iter()
            .map(|row| {
                (
                    row.zone().slug().as_str(),
                    row.dryness().map(|dryness| dryness.value()),
                    row.dryness_with().map(|dryness| dryness.value()),
                    *row.change(),
                )
            })
            .collect();

        assert_eq!(
            rows,
            vec![
                ("kifri", Some(50), Some(50), Some(0)),
                ("chamchamal", Some(60), None, None),
                ("kalar", None, Some(70), None),
                ("penjwen", None, None, None),
            ],
            "no change at all still ranks above a change that cannot be known"
        );
    }

    #[test]
    fn the_region_line_has_every_year_with_data_oldest_first() {
        let compared = RegionComparison::build(
            comparison(),
            zones(),
            &[
                reading(1, "2026-03", 60),
                reading(2, "2026-03", 31),
                reading(1, "2024-03", 40),
                reading(1, "2025-03", 55),
                // Another calendar month is not part of this line.
                reading(1, "2025-04", 100),
            ],
        );

        assert_eq!(
            compared
                .region()
                .iter()
                .map(|point| (*point.year(), *point.average_dryness()))
                .collect::<Vec<_>>(),
            vec![(2024, 40.0), (2025, 55.0), (2026, 45.5)]
        );
    }

    #[test]
    fn nothing_measured_gives_every_zone_with_nothing_and_an_empty_region_line() {
        let compared = RegionComparison::build(comparison(), zones(), &[]);

        assert_eq!(compared.zones().len(), 4);
        assert!(compared.zones().iter().all(|row| row.change().is_none()));
        assert!(compared.region().is_empty());
    }
}
