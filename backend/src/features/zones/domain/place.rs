use getset::Getters;

use crate::features::zones::domain::{Shape, SubZone, Zone, ZoneSlug};

/// The Sorani names of the governorates, as written on the team's map
/// (`web/map_demo/kri_map_data.js`). A zone carries its governorate's
/// English name only.
const GOVERNORATE_NAMES_KU: [(&str, &str); 4] = [
    ("Duhok", "دهۆک"),
    ("Erbil", "هەولێر"),
    ("Sulaymaniyah", "سلێمانی"),
    ("Halabja", "هەڵەبجە"),
];

/// A governorate, known only through the zones that name it.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct Governorate {
    name_en: String,
    name_ku: String,
}

impl Governorate {
    /// A governorate this list does not know yet shows its English name in
    /// both languages rather than nothing.
    pub fn named(name_en: &str) -> Self {
        let name_ku = GOVERNORATE_NAMES_KU
            .iter()
            .find(|(known, _)| *known == name_en)
            .map_or(name_en, |(_, name_ku)| name_ku);

        Self {
            name_en: name_en.to_string(),
            name_ku: name_ku.to_string(),
        }
    }

    /// The governorates the zones lie in, each once, in the order the zones
    /// first name them.
    pub fn of(zones: &[Zone]) -> Vec<Self> {
        let mut governorates: Vec<Self> = Vec::new();

        for zone in zones {
            if !governorates
                .iter()
                .any(|known| known.name_en() == zone.governorate())
            {
                governorates.push(Self::named(zone.governorate()));
            }
        }

        governorates
    }
}

/// Where a point is: the sub-zone it lies in, with that sub-zone's zone and
/// governorate.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct Place {
    governorate: String,
    zone_slug: ZoneSlug,
    sub_zone_slug: ZoneSlug,
}

/// Every sub-zone shape with the place it stands for, held in memory to
/// answer which place a point is in. The shapes are reference data that
/// change only by migration.
#[derive(Clone, Debug, Default)]
pub struct PlaceIndex {
    entries: Vec<(Shape, Place)>,
}

impl PlaceIndex {
    /// A sub-zone whose zone is not among `zones` is left out: there would
    /// be no governorate to give its points.
    pub fn new(zones: &[Zone], shapes: Vec<(SubZone, Shape)>) -> Self {
        let entries = shapes
            .into_iter()
            .filter_map(|(sub_zone, shape)| {
                let zone = zones.iter().find(|zone| zone.id() == sub_zone.zone_id())?;

                Some((
                    shape,
                    Place {
                        governorate: zone.governorate().clone(),
                        zone_slug: zone.slug().clone(),
                        sub_zone_slug: sub_zone.slug().clone(),
                    },
                ))
            })
            .collect();

        Self { entries }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The place the point is in, or `None` when it is outside every shape.
    /// Should two shapes overlap, the one stored first wins, so the same
    /// point always gets the same place.
    pub fn locate(&self, lat: f64, lon: f64) -> Option<&Place> {
        self.entries
            .iter()
            .find(|(shape, _)| shape.contains(lon, lat))
            .map(|(_, place)| place)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slug(value: &str) -> ZoneSlug {
        ZoneSlug::new(value.to_string()).expect("slug")
    }

    fn zone(id: i32, name: &str, governorate: &str) -> Zone {
        Zone::rehydrate(
            id,
            slug(&name.to_lowercase()),
            name.to_string(),
            name.to_string(),
            governorate.to_string(),
        )
    }

    fn sub_zone(id: i32, zone_id: i32, name: &str) -> SubZone {
        SubZone::rehydrate(
            id,
            zone_id,
            slug(&name.to_lowercase()),
            name.to_string(),
            name.to_string(),
        )
    }

    fn square(west: f64, south: f64) -> Shape {
        Shape::new(vec![vec![
            (west, south),
            (west + 1.0, south),
            (west + 1.0, south + 1.0),
            (west, south + 1.0),
        ]])
        .expect("shape")
    }

    /// Two sub-zones of Kalar side by side, and one of Erbil to their north.
    fn index() -> PlaceIndex {
        PlaceIndex::new(
            &[zone(1, "Kalar", "Sulaymaniyah"), zone(2, "Erbil", "Erbil")],
            vec![
                (sub_zone(1, 1, "West"), square(44.0, 35.0)),
                (sub_zone(2, 1, "East"), square(45.0, 35.0)),
                (sub_zone(3, 2, "North"), square(44.0, 36.0)),
            ],
        )
    }

    fn found(lat: f64, lon: f64) -> Option<(String, String, String)> {
        index().locate(lat, lon).map(|place| {
            (
                place.governorate().clone(),
                place.zone_slug().as_str().to_string(),
                place.sub_zone_slug().as_str().to_string(),
            )
        })
    }

    #[test]
    fn a_point_gets_the_sub_zone_it_is_in_with_its_zone_and_governorate() {
        assert_eq!(
            found(35.5, 45.5),
            Some((
                "Sulaymaniyah".to_string(),
                "kalar".to_string(),
                "east".to_string()
            ))
        );
        assert_eq!(
            found(36.5, 44.5),
            Some((
                "Erbil".to_string(),
                "erbil".to_string(),
                "north".to_string()
            ))
        );
    }

    #[test]
    fn a_point_outside_every_shape_has_no_place() {
        assert_eq!(found(38.0, 44.5), None);
        assert_eq!(found(36.5, 45.5), None, "the corner no sub-zone covers");
    }

    #[test]
    fn latitude_comes_first_and_is_not_mistaken_for_longitude() {
        assert_eq!(found(44.5, 35.5), None);
    }

    #[test]
    fn a_point_on_a_shared_border_gets_one_place_and_always_the_same() {
        let first = found(35.5, 45.0);

        assert_eq!(
            first.as_ref().map(|(_, _, sub_zone)| sub_zone.as_str()),
            Some("east")
        );
        assert_eq!(found(35.5, 45.0), first);
    }

    #[test]
    fn a_sub_zone_without_its_zone_is_left_out() {
        let index = PlaceIndex::new(
            &[zone(1, "Kalar", "Sulaymaniyah")],
            vec![(sub_zone(3, 2, "North"), square(44.0, 36.0))],
        );

        assert!(index.is_empty());
        assert_eq!(index.locate(36.5, 44.5), None);
    }

    #[test]
    fn the_governorates_are_listed_once_each_in_zone_order_with_their_sorani_names() {
        let governorates = Governorate::of(&[
            zone(1, "Akre", "Duhok"),
            zone(2, "Zakho", "Duhok"),
            zone(3, "Koya", "Erbil"),
            zone(4, "Kalar", "Sulaymaniyah"),
            zone(5, "Khurmal", "Halabja"),
        ]);

        let names: Vec<(&str, &str)> = governorates
            .iter()
            .map(|one| (one.name_en().as_str(), one.name_ku().as_str()))
            .collect();

        assert_eq!(
            names,
            vec![
                ("Duhok", "دهۆک"),
                ("Erbil", "هەولێر"),
                ("Sulaymaniyah", "سلێمانی"),
                ("Halabja", "هەڵەبجە"),
            ]
        );
    }

    #[test]
    fn a_governorate_without_a_known_sorani_name_shows_its_english_one() {
        assert_eq!(Governorate::named("Garmiyan").name_ku(), "Garmiyan");
    }
}
