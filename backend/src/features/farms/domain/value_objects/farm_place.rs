use getset::Getters;

use crate::{features::farms::domain::FarmError, shared::DomainError};

/// Where a farm lies: the governorate by its English name, and the district
/// and sub-district by their slugs, as the zones feature names them. A farm
/// has all three or, when it is outside every sub-district, none.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Getters)]
#[getset(get = "pub")]
pub struct FarmPlace {
    governorate: String,
    zone_slug: String,
    sub_zone_slug: String,
}

impl FarmPlace {
    pub fn new(
        governorate: String,
        zone_slug: String,
        sub_zone_slug: String,
    ) -> Result<Self, FarmError> {
        if [&governorate, &zone_slug, &sub_zone_slug]
            .iter()
            .any(|part| part.trim().is_empty())
        {
            return Err(DomainError::InvalidValue(
                "A farm's place needs a governorate, a zone and a sub-zone".to_string(),
            )
            .into());
        }

        Ok(Self {
            governorate,
            zone_slug,
            sub_zone_slug,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_place_names_its_governorate_zone_and_sub_zone() {
        let place = FarmPlace::new(
            "Sulaymaniyah".to_string(),
            "chamchamal".to_string(),
            "sangaw".to_string(),
        )
        .expect("place");

        assert_eq!(place.governorate(), "Sulaymaniyah");
        assert_eq!(place.zone_slug(), "chamchamal");
        assert_eq!(place.sub_zone_slug(), "sangaw");
    }

    #[test]
    fn a_place_missing_one_of_its_three_parts_is_refused() {
        for (governorate, zone, sub_zone) in [
            ("", "chamchamal", "sangaw"),
            ("Sulaymaniyah", " ", "sangaw"),
            ("Sulaymaniyah", "chamchamal", ""),
        ] {
            assert!(
                FarmPlace::new(
                    governorate.to_string(),
                    zone.to_string(),
                    sub_zone.to_string()
                )
                .is_err(),
                "{governorate:?} {zone:?} {sub_zone:?}"
            );
        }
    }
}
