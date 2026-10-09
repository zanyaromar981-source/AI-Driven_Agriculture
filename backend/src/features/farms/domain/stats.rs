use chrono::{DateTime, Utc};
use getset::{CopyGetters, Getters};

use crate::features::farms::domain::{Crop, GridCell, UNKNOWN_AREA};

const UNKNOWN_NAME_EN: &str = "Unknown";
const UNKNOWN_NAME_KU: &str = "نەزانراو";

/// How finely farms are grouped: all together, or by one kind of area.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AreaLevel {
    Region,
    Governorate,
    Zone,
    SubZone,
}

/// Which area a row of sums is about. A level names the parts down to its
/// own and leaves the finer ones `None`; all three are `None` for the whole
/// region and for the farms that have no place.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct AreaKey {
    pub governorate: Option<String>,
    pub zone_slug: Option<String>,
    pub sub_zone_slug: Option<String>,
}

/// The farms of one area, added up by the repository.
#[derive(Clone, Debug, PartialEq)]
pub struct AreaCount {
    pub level: AreaLevel,
    pub key: AreaKey,
    pub farms: u64,
    /// Different owners: a farmer with two farms in the area counts once.
    pub farmers: u64,
    /// The summed area inside the farms' outlines.
    pub area_m2: f64,
    /// When a farm of the area was last written.
    pub latest_change: Option<DateTime<Utc>>,
}

/// One crop on the farms of one area, added up by the repository.
#[derive(Clone, Debug, PartialEq)]
pub struct AreaCropSum {
    pub level: AreaLevel,
    pub key: AreaKey,
    pub crop: Crop,
    /// The summed `inside_pct` of the crop's cells.
    pub inside_pct: f64,
    pub farms: u64,
    pub farmers: u64,
}

/// The name of a governorate as the zones feature gives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GovernorateName {
    pub name_en: String,
    pub name_ku: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZoneName {
    pub slug: String,
    pub name_en: String,
    pub name_ku: String,
}

/// A sub-zone slug is unique inside its zone only, so the name carries the
/// zone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubZoneName {
    pub zone_slug: String,
    pub slug: String,
    pub name_en: String,
    pub name_ku: String,
}

/// What every area is called, each list in the order a report shows it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AreaNames {
    pub governorates: Vec<GovernorateName>,
    pub zones: Vec<ZoneName>,
    pub sub_zones: Vec<SubZoneName>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct FarmTotals {
    farmers: u64,
    farms: u64,
    /// The land inside the farms' outlines, painted or not.
    dunam: f64,
}

/// The land under one crop, and how many farms and farmers grow it.
#[derive(Clone, Copy, Debug, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct CropTotals {
    crop: Crop,
    dunam: f64,
    farms: u64,
    farmers: u64,
}

/// The farms of one governorate, zone or sub-zone, or of no place at all.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct AreaStats {
    /// `unknown` for the farms with no place.
    slug: String,
    name_en: String,
    name_ku: String,
    /// The governorate a zone or a sub-zone lies in.
    governorate: Option<String>,
    /// The zone a sub-zone lies in.
    zone_slug: Option<String>,
    totals: FarmTotals,
    /// Largest first.
    crops: Vec<CropTotals>,
}

/// Farms, farmers and land added up for reports and charts.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct FarmStats {
    /// When a farm counted here was last written, or the time of asking
    /// when none is counted.
    as_of: DateTime<Utc>,
    totals: FarmTotals,
    by_governorate: Vec<AreaStats>,
    by_zone: Vec<AreaStats>,
    by_sub_zone: Vec<AreaStats>,
    by_crop: Vec<CropTotals>,
}

impl FarmStats {
    /// Puts the repository's sums into report order and gives each area its
    /// names. Areas come in the order of `names`, an area the names do not
    /// list after them under its slug, and the farms with no place last.
    /// Crops come largest first.
    pub fn assemble(
        counts: Vec<AreaCount>,
        crop_sums: Vec<AreaCropSum>,
        names: &AreaNames,
        now: DateTime<Utc>,
    ) -> Self {
        let region = counts.iter().find(|count| count.level == AreaLevel::Region);

        let areas = |level: AreaLevel| -> Vec<AreaStats> {
            let mut ranked: Vec<(usize, AreaStats)> = counts
                .iter()
                .filter(|count| count.level == level)
                .map(|count| {
                    let (rank, named) = named_area(level, &count.key, names);

                    (
                        rank,
                        AreaStats {
                            totals: totals(count),
                            crops: crop_totals(
                                crop_sums
                                    .iter()
                                    .filter(|sum| sum.level == level && sum.key == count.key),
                            ),
                            ..named
                        },
                    )
                })
                .collect();

            ranked.sort_by(|(first_rank, first), (second_rank, second)| {
                first_rank
                    .cmp(second_rank)
                    .then_with(|| first.slug.cmp(&second.slug))
            });

            ranked.into_iter().map(|(_, area)| area).collect()
        };

        Self {
            as_of: region.and_then(|count| count.latest_change).unwrap_or(now),
            totals: region.map(totals).unwrap_or_default(),
            by_governorate: areas(AreaLevel::Governorate),
            by_zone: areas(AreaLevel::Zone),
            by_sub_zone: areas(AreaLevel::SubZone),
            by_crop: crop_totals(
                crop_sums
                    .iter()
                    .filter(|sum| sum.level == AreaLevel::Region),
            ),
        }
    }
}

fn totals(count: &AreaCount) -> FarmTotals {
    FarmTotals {
        farmers: count.farmers,
        farms: count.farms,
        dunam: count.area_m2 / (GridCell::AREA_M2 * GridCell::PER_DUNAM),
    }
}

/// Largest area first; ties keep the order the crop codes are declared in.
/// `Empty` is never a crop.
fn crop_totals<'a>(sums: impl Iterator<Item = &'a AreaCropSum>) -> Vec<CropTotals> {
    let declared = |crop: &Crop| Crop::ALL.iter().position(|other| other == crop);

    let mut crops: Vec<CropTotals> = sums
        .filter(|sum| sum.crop != Crop::Empty && sum.inside_pct > 0.0)
        .map(|sum| CropTotals {
            crop: sum.crop,
            dunam: GridCell::dunams(sum.inside_pct),
            farms: sum.farms,
            farmers: sum.farmers,
        })
        .collect();

    crops.sort_by(|first, second| {
        second
            .dunam
            .total_cmp(&first.dunam)
            .then(declared(&first.crop).cmp(&declared(&second.crop)))
    });

    crops
}

/// The area a key stands for at a level, without its sums, and its place in
/// the report: the position of its name, then the unnamed, then `unknown`.
fn named_area(level: AreaLevel, key: &AreaKey, names: &AreaNames) -> (usize, AreaStats) {
    let area = |slug: &str, name_en: &str, name_ku: &str| AreaStats {
        slug: slug.to_string(),
        name_en: name_en.to_string(),
        name_ku: name_ku.to_string(),
        governorate: match level {
            AreaLevel::Zone | AreaLevel::SubZone => key.governorate.clone(),
            AreaLevel::Region | AreaLevel::Governorate => None,
        },
        zone_slug: match level {
            AreaLevel::SubZone => key.zone_slug.clone(),
            _ => None,
        },
        totals: FarmTotals::default(),
        crops: Vec::new(),
    };

    let unnamed = usize::MAX - 1;
    let unknown = (
        usize::MAX,
        area(UNKNOWN_AREA, UNKNOWN_NAME_EN, UNKNOWN_NAME_KU),
    );

    match level {
        AreaLevel::Region => unknown,
        AreaLevel::Governorate => {
            let Some(governorate) = &key.governorate else {
                return unknown;
            };

            let slug = governorate.to_lowercase().replace(' ', "-");

            match names
                .governorates
                .iter()
                .position(|name| &name.name_en == governorate)
            {
                Some(rank) => (
                    rank,
                    area(&slug, governorate, &names.governorates[rank].name_ku),
                ),
                None => (unnamed, area(&slug, governorate, governorate)),
            }
        }
        AreaLevel::Zone => {
            let Some(zone_slug) = &key.zone_slug else {
                return unknown;
            };

            match names.zones.iter().position(|name| &name.slug == zone_slug) {
                Some(rank) => {
                    let name = &names.zones[rank];

                    (rank, area(zone_slug, &name.name_en, &name.name_ku))
                }
                None => (unnamed, area(zone_slug, zone_slug, zone_slug)),
            }
        }
        AreaLevel::SubZone => {
            let (Some(zone_slug), Some(sub_zone_slug)) = (&key.zone_slug, &key.sub_zone_slug)
            else {
                return unknown;
            };

            match names
                .sub_zones
                .iter()
                .position(|name| &name.zone_slug == zone_slug && &name.slug == sub_zone_slug)
            {
                Some(rank) => {
                    let name = &names.sub_zones[rank];

                    (rank, area(sub_zone_slug, &name.name_en, &name.name_ku))
                }
                None => (unnamed, area(sub_zone_slug, sub_zone_slug, sub_zone_slug)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn names() -> AreaNames {
        AreaNames {
            governorates: vec![
                GovernorateName {
                    name_en: "Erbil".to_string(),
                    name_ku: "هەولێر".to_string(),
                },
                GovernorateName {
                    name_en: "Sulaymaniyah".to_string(),
                    name_ku: "سلێمانی".to_string(),
                },
            ],
            zones: vec![
                ZoneName {
                    slug: "koya".to_string(),
                    name_en: "Koya".to_string(),
                    name_ku: "کۆیە".to_string(),
                },
                ZoneName {
                    slug: "kalar".to_string(),
                    name_en: "Kalar".to_string(),
                    name_ku: "کەلار".to_string(),
                },
            ],
            sub_zones: vec![
                SubZoneName {
                    zone_slug: "koya".to_string(),
                    slug: "shorsh".to_string(),
                    name_en: "Shorsh".to_string(),
                    name_ku: "شۆڕش".to_string(),
                },
                SubZoneName {
                    zone_slug: "kalar".to_string(),
                    slug: "markaz-kalar".to_string(),
                    name_en: "Markaz Kalar".to_string(),
                    name_ku: "ناوەندی کەلار".to_string(),
                },
            ],
        }
    }

    fn key(governorate: Option<&str>, zone: Option<&str>, sub_zone: Option<&str>) -> AreaKey {
        AreaKey {
            governorate: governorate.map(str::to_string),
            zone_slug: zone.map(str::to_string),
            sub_zone_slug: sub_zone.map(str::to_string),
        }
    }

    fn count(level: AreaLevel, key: AreaKey, farms: u64, farmers: u64, dunam: f64) -> AreaCount {
        AreaCount {
            level,
            key,
            farms,
            farmers,
            area_m2: dunam * 2_500.0,
            latest_change: None,
        }
    }

    fn crop(level: AreaLevel, key: AreaKey, crop: Crop, dunam: f64, farms: u64) -> AreaCropSum {
        AreaCropSum {
            level,
            key,
            crop,
            inside_pct: dunam * 2_500.0,
            farms,
            farmers: farms,
        }
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 9, 12, 0, 0).unwrap()
    }

    /// Two farms of one farmer in Kalar, one farm in Koya, one with no place.
    fn counts() -> Vec<AreaCount> {
        vec![
            count(AreaLevel::Region, key(None, None, None), 4, 3, 100.0),
            count(
                AreaLevel::Governorate,
                key(Some("Sulaymaniyah"), None, None),
                2,
                1,
                50.0,
            ),
            count(AreaLevel::Governorate, key(None, None, None), 1, 1, 20.0),
            count(
                AreaLevel::Governorate,
                key(Some("Erbil"), None, None),
                1,
                1,
                30.0,
            ),
            count(
                AreaLevel::Zone,
                key(Some("Sulaymaniyah"), Some("kalar"), None),
                2,
                1,
                50.0,
            ),
            count(AreaLevel::Zone, key(None, None, None), 1, 1, 20.0),
            count(
                AreaLevel::Zone,
                key(Some("Erbil"), Some("koya"), None),
                1,
                1,
                30.0,
            ),
            count(
                AreaLevel::SubZone,
                key(Some("Sulaymaniyah"), Some("kalar"), Some("markaz-kalar")),
                2,
                1,
                50.0,
            ),
            count(
                AreaLevel::SubZone,
                key(Some("Erbil"), Some("koya"), Some("shorsh")),
                1,
                1,
                30.0,
            ),
            count(AreaLevel::SubZone, key(None, None, None), 1, 1, 20.0),
        ]
    }

    fn slugs(areas: &[AreaStats]) -> Vec<&str> {
        areas.iter().map(|area| area.slug().as_str()).collect()
    }

    #[test]
    fn the_totals_are_the_regions_row_and_a_dunam_is_2500_square_metres() {
        let stats = FarmStats::assemble(counts(), vec![], &names(), now());

        assert_eq!(stats.totals().farms(), 4);
        assert_eq!(stats.totals().farmers(), 3);
        assert_eq!(stats.totals().dunam(), 100.0);
    }

    #[test]
    fn areas_come_in_the_order_of_their_names_with_the_placeless_farms_last() {
        let stats = FarmStats::assemble(counts(), vec![], &names(), now());

        assert_eq!(
            slugs(stats.by_governorate()),
            vec!["erbil", "sulaymaniyah", "unknown"]
        );
        assert_eq!(slugs(stats.by_zone()), vec!["koya", "kalar", "unknown"]);
        assert_eq!(
            slugs(stats.by_sub_zone()),
            vec!["shorsh", "markaz-kalar", "unknown"]
        );
    }

    #[test]
    fn an_area_carries_both_names_its_parents_and_its_own_sums() {
        let stats = FarmStats::assemble(counts(), vec![], &names(), now());

        let governorate = &stats.by_governorate()[1];
        assert_eq!(governorate.name_en(), "Sulaymaniyah");
        assert_eq!(governorate.name_ku(), "سلێمانی");
        assert_eq!(governorate.governorate(), &None);
        assert_eq!(governorate.totals().farms(), 2);
        assert_eq!(
            governorate.totals().farmers(),
            1,
            "one farmer with two farms in the area is one farmer"
        );
        assert_eq!(governorate.totals().dunam(), 50.0);

        let zone = &stats.by_zone()[1];
        assert_eq!(zone.name_ku(), "کەلار");
        assert_eq!(zone.governorate(), &Some("Sulaymaniyah".to_string()));
        assert_eq!(zone.zone_slug(), &None);

        let sub_zone = &stats.by_sub_zone()[1];
        assert_eq!(sub_zone.name_en(), "Markaz Kalar");
        assert_eq!(sub_zone.governorate(), &Some("Sulaymaniyah".to_string()));
        assert_eq!(sub_zone.zone_slug(), &Some("kalar".to_string()));
    }

    #[test]
    fn the_farms_with_no_place_are_one_row_named_unknown_in_both_languages() {
        let stats = FarmStats::assemble(counts(), vec![], &names(), now());

        let unknown = stats.by_zone().last().expect("row");

        assert_eq!(unknown.slug(), "unknown");
        assert_eq!(unknown.name_en(), "Unknown");
        assert_eq!(unknown.name_ku(), "نەزانراو");
        assert_eq!(unknown.totals().farms(), 1);
        assert_eq!(unknown.totals().dunam(), 20.0);
    }

    #[test]
    fn an_area_the_names_do_not_list_is_shown_under_its_slug_before_unknown() {
        let mut counts = counts();
        counts.push(count(
            AreaLevel::Zone,
            key(Some("Erbil"), Some("old-zone"), None),
            1,
            1,
            5.0,
        ));

        let stats = FarmStats::assemble(counts, vec![], &names(), now());

        assert_eq!(
            slugs(stats.by_zone()),
            vec!["koya", "kalar", "old-zone", "unknown"]
        );
        assert_eq!(stats.by_zone()[2].name_en(), "old-zone");
        assert_eq!(stats.by_zone()[2].name_ku(), "old-zone");
    }

    #[test]
    fn crops_come_largest_first_and_empty_land_is_not_a_crop() {
        let region = key(None, None, None);
        let sums = vec![
            crop(AreaLevel::Region, region.clone(), Crop::Tomato, 10.0, 1),
            crop(AreaLevel::Region, region.clone(), Crop::Empty, 40.0, 4),
            crop(AreaLevel::Region, region.clone(), Crop::Wheat, 50.0, 3),
            crop(AreaLevel::Region, region, Crop::Barley, 10.0, 2),
        ];

        let stats = FarmStats::assemble(counts(), sums, &names(), now());

        assert_eq!(
            stats
                .by_crop()
                .iter()
                .map(|crop| (crop.crop(), crop.dunam(), crop.farms()))
                .collect::<Vec<_>>(),
            vec![
                (Crop::Wheat, 50.0, 3),
                (Crop::Barley, 10.0, 2),
                (Crop::Tomato, 10.0, 1),
            ],
            "a tie keeps the order the crops are declared in"
        );
    }

    #[test]
    fn each_area_gets_only_the_crops_summed_for_it_at_its_own_level() {
        let kalar = key(Some("Sulaymaniyah"), Some("kalar"), None);
        let sums = vec![
            crop(AreaLevel::Zone, kalar.clone(), Crop::Wheat, 30.0, 2),
            crop(
                AreaLevel::Zone,
                key(Some("Erbil"), Some("koya"), None),
                Crop::Barley,
                12.0,
                1,
            ),
            crop(
                AreaLevel::Governorate,
                key(Some("Sulaymaniyah"), None, None),
                Crop::Wheat,
                30.0,
                2,
            ),
            crop(AreaLevel::Zone, key(None, None, None), Crop::Onion, 4.0, 1),
        ];

        let stats = FarmStats::assemble(counts(), sums, &names(), now());

        let crops = |area: &AreaStats| -> Vec<(Crop, f64)> {
            area.crops()
                .iter()
                .map(|crop| (crop.crop(), crop.dunam()))
                .collect()
        };

        assert_eq!(crops(&stats.by_zone()[0]), vec![(Crop::Barley, 12.0)]);
        assert_eq!(crops(&stats.by_zone()[1]), vec![(Crop::Wheat, 30.0)]);
        assert_eq!(crops(&stats.by_zone()[2]), vec![(Crop::Onion, 4.0)]);
        assert_eq!(crops(&stats.by_governorate()[1]), vec![(Crop::Wheat, 30.0)]);
        assert!(crops(&stats.by_governorate()[0]).is_empty());
    }

    #[test]
    fn as_of_is_the_last_write_to_a_counted_farm() {
        let written = Utc.with_ymd_and_hms(2026, 10, 1, 8, 30, 0).unwrap();
        let mut counts = counts();
        counts[0].latest_change = Some(written);

        let stats = FarmStats::assemble(counts, vec![], &names(), now());

        assert_eq!(*stats.as_of(), written);
    }

    #[test]
    fn with_no_farms_everything_is_zero_or_empty_and_as_of_is_now() {
        let stats = FarmStats::assemble(vec![], vec![], &names(), now());

        assert_eq!(stats.totals(), &FarmTotals::default());
        assert!(stats.by_governorate().is_empty());
        assert!(stats.by_zone().is_empty());
        assert!(stats.by_sub_zone().is_empty());
        assert!(stats.by_crop().is_empty());
        assert_eq!(*stats.as_of(), now());
    }
}
