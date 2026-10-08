use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use getset::Getters;

use crate::{
    features::fires::domain::{
        ExternalId, FireError, FireLocation, FireSource, FireStatus, PlaceName, WindDirection,
        ZoneSlug,
    },
    shared::DomainError,
};

const MAX_WIND_KMH: f64 = 300.0;

/// One fire a satellite detected, as the data job last described it. Every
/// number comes from the job; a number it does not know is `None`, never 0.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Fire {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    external_id: ExternalId,
    location: FireLocation,
    zone_slug: Option<ZoneSlug>,
    place_en: Option<PlaceName>,
    place_ku: Option<PlaceName>,
    detected_at: DateTime<Utc>,
    area_ha: Option<f64>,
    wind_kmh: Option<f64>,
    wind_direction: Option<WindDirection>,
    status: FireStatus,
    farms_within_5km: Option<i32>,
    farmers_alerted: Option<i32>,
    source: FireSource,
    updated_at: DateTime<Utc>,
}

impl Fire {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        external_id: ExternalId,
        location: FireLocation,
        zone_slug: Option<ZoneSlug>,
        place_en: Option<PlaceName>,
        place_ku: Option<PlaceName>,
        detected_at: DateTime<Utc>,
        area_ha: Option<f64>,
        wind_kmh: Option<f64>,
        wind_direction: Option<WindDirection>,
        status: FireStatus,
        farms_within_5km: Option<i32>,
        farmers_alerted: Option<i32>,
        source: FireSource,
    ) -> Result<Self, FireError> {
        if area_ha.is_some_and(|area| !area.is_finite() || area < 0.0) {
            return Err(invalid("Burnt area must be zero or more hectares"));
        }

        if wind_kmh.is_some_and(|wind| !(0.0..=MAX_WIND_KMH).contains(&wind)) {
            return Err(invalid("Wind speed must be between 0 and 300 km/h"));
        }

        if farms_within_5km.is_some_and(|farms| farms < 0) {
            return Err(invalid("Farms within 5 km must be zero or more"));
        }

        if farmers_alerted.is_some_and(|farmers| farmers < 0) {
            return Err(invalid("Farmers alerted must be zero or more"));
        }

        Ok(Self {
            id: None,
            external_id,
            location,
            zone_slug,
            place_en,
            place_ku,
            detected_at,
            area_ha,
            wind_kmh,
            wind_direction,
            status,
            farms_within_5km,
            farmers_alerted,
            source,
            updated_at: Utc::now(),
        })
    }

    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        external_id: ExternalId,
        location: FireLocation,
        zone_slug: Option<ZoneSlug>,
        place_en: Option<PlaceName>,
        place_ku: Option<PlaceName>,
        detected_at: DateTime<Utc>,
        area_ha: Option<f64>,
        wind_kmh: Option<f64>,
        wind_direction: Option<WindDirection>,
        status: FireStatus,
        farms_within_5km: Option<i32>,
        farmers_alerted: Option<i32>,
        source: FireSource,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            external_id,
            location,
            zone_slug,
            place_en,
            place_ku,
            detected_at,
            area_ha,
            wind_kmh,
            wind_direction,
            status,
            farms_within_5km,
            farmers_alerted,
            source,
            updated_at,
        }
    }
}

fn invalid(detail: &str) -> FireError {
    DomainError::InvalidValue(detail.to_string()).into()
}

/// The totals the dashboard shows above the list of fires.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct FireSummary {
    /// Fires that are active or spreading.
    active: usize,
    under_control: usize,
    area_ha: f64,
    farms_within_5km: i64,
    farmers_alerted: i64,
    /// Zones with an active or spreading fire, each once, in alphabetical
    /// order.
    zones: Vec<ZoneSlug>,
}

impl FireSummary {
    /// Adds up the given fires. A fire that is out no longer threatens
    /// anything, so it adds nothing to the area or to the two counts; a
    /// number the job did not send counts as 0.
    pub fn of(fires: &[Fire]) -> Self {
        let not_out = || fires.iter().filter(|fire| !fire.status.is_out());

        let zones: BTreeSet<ZoneSlug> = fires
            .iter()
            .filter(|fire| fire.status.is_burning())
            .filter_map(|fire| fire.zone_slug.clone())
            .collect();

        Self {
            active: fires.iter().filter(|fire| fire.status.is_burning()).count(),
            under_control: fires
                .iter()
                .filter(|fire| fire.status == FireStatus::UnderControl)
                .count(),
            // Folded from 0.0 because the sum of no floats is -0.0, which
            // would reach the dashboard as "-0.0".
            area_ha: not_out()
                .map(|fire| fire.area_ha.unwrap_or(0.0))
                .fold(0.0, |total, area| total + area),
            farms_within_5km: not_out()
                .map(|fire| i64::from(fire.farms_within_5km.unwrap_or(0)))
                .sum(),
            farmers_alerted: not_out()
                .map(|fire| i64::from(fire.farmers_alerted.unwrap_or(0)))
                .sum(),
            zones: zones.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Numbers {
        area_ha: Option<f64>,
        wind_kmh: Option<f64>,
        farms_within_5km: Option<i32>,
        farmers_alerted: Option<i32>,
    }

    impl Default for Numbers {
        fn default() -> Self {
            Self {
                area_ha: Some(12.5),
                wind_kmh: Some(18.0),
                farms_within_5km: Some(4),
                farmers_alerted: Some(9),
            }
        }
    }

    fn fire_with(
        status: FireStatus,
        zone: Option<&str>,
        numbers: Numbers,
    ) -> Result<Fire, FireError> {
        Fire::new(
            ExternalId::new("firms-1".to_string()).expect("external id"),
            FireLocation::new(35.53, 44.83).expect("location"),
            zone.map(|slug| ZoneSlug::new(slug.to_string()).expect("zone")),
            None,
            None,
            Utc::now(),
            numbers.area_ha,
            numbers.wind_kmh,
            Some(WindDirection::Ne),
            status,
            numbers.farms_within_5km,
            numbers.farmers_alerted,
            FireSource::new("NASA FIRMS".to_string()).expect("source"),
        )
    }

    fn fire(status: FireStatus, zone: Option<&str>) -> Fire {
        fire_with(status, zone, Numbers::default()).expect("fire")
    }

    #[test]
    fn a_new_fire_has_no_id_until_it_is_stored() {
        assert!(fire(FireStatus::Active, None).id().is_none());
    }

    #[test]
    fn a_fire_may_come_with_none_of_the_optional_numbers() {
        let bare = fire_with(
            FireStatus::Active,
            None,
            Numbers {
                area_ha: None,
                wind_kmh: None,
                farms_within_5km: None,
                farmers_alerted: None,
            },
        );

        assert!(bare.is_ok(), "a detection is often a position and a time");
    }

    #[test]
    fn a_negative_or_non_finite_area_is_refused() {
        for area in [-0.1, f64::NAN, f64::INFINITY] {
            let result = fire_with(
                FireStatus::Active,
                None,
                Numbers {
                    area_ha: Some(area),
                    ..Numbers::default()
                },
            );

            assert!(result.is_err(), "{area} hectares was accepted");
        }
    }

    #[test]
    fn the_wind_speed_is_between_zero_and_three_hundred() {
        let with_wind = |wind: f64| {
            fire_with(
                FireStatus::Active,
                None,
                Numbers {
                    wind_kmh: Some(wind),
                    ..Numbers::default()
                },
            )
        };

        assert!(with_wind(0.0).is_ok());
        assert!(with_wind(300.0).is_ok());
        assert!(with_wind(-1.0).is_err());
        assert!(with_wind(300.1).is_err());
        assert!(with_wind(f64::NAN).is_err());
    }

    #[test]
    fn the_two_counts_cannot_be_negative() {
        let farms = fire_with(
            FireStatus::Active,
            None,
            Numbers {
                farms_within_5km: Some(-1),
                ..Numbers::default()
            },
        );
        let farmers = fire_with(
            FireStatus::Active,
            None,
            Numbers {
                farmers_alerted: Some(-1),
                ..Numbers::default()
            },
        );

        assert!(farms.is_err());
        assert!(farmers.is_err());
    }

    #[test]
    fn the_summary_counts_active_and_spreading_together() {
        let summary = FireSummary::of(&[
            fire(FireStatus::Active, None),
            fire(FireStatus::Spreading, None),
            fire(FireStatus::UnderControl, None),
            fire(FireStatus::Out, None),
        ]);

        assert_eq!(*summary.active(), 2);
        assert_eq!(*summary.under_control(), 1);
    }

    #[test]
    fn a_fire_that_is_out_adds_nothing_to_the_totals() {
        let summary = FireSummary::of(&[
            fire(FireStatus::Active, None),
            fire(FireStatus::UnderControl, None),
            fire(FireStatus::Out, None),
        ]);

        assert_eq!(*summary.area_ha(), 25.0);
        assert_eq!(*summary.farms_within_5km(), 8);
        assert_eq!(*summary.farmers_alerted(), 18);
    }

    #[test]
    fn a_number_the_job_did_not_send_counts_as_zero() {
        let unknown = fire_with(
            FireStatus::Active,
            None,
            Numbers {
                area_ha: None,
                wind_kmh: None,
                farms_within_5km: None,
                farmers_alerted: None,
            },
        )
        .expect("fire");

        let summary = FireSummary::of(&[unknown, fire(FireStatus::Spreading, None)]);

        assert_eq!(*summary.area_ha(), 12.5);
        assert_eq!(*summary.farms_within_5km(), 4);
        assert_eq!(*summary.farmers_alerted(), 9);
    }

    #[test]
    fn the_zones_are_those_still_burning_each_named_once() {
        let summary = FireSummary::of(&[
            fire(FireStatus::Spreading, Some("soran")),
            fire(FireStatus::Active, Some("chamchamal")),
            fire(FireStatus::Active, Some("soran")),
            fire(FireStatus::Active, None),
            fire(FireStatus::UnderControl, Some("akre")),
            fire(FireStatus::Out, Some("zakho")),
        ]);

        let zones: Vec<&str> = summary.zones().iter().map(ZoneSlug::as_str).collect();

        assert_eq!(zones, vec!["chamchamal", "soran"]);
    }

    #[test]
    fn no_fires_is_a_summary_of_zeros() {
        let summary = FireSummary::of(&[]);

        assert_eq!(*summary.active(), 0);
        assert_eq!(*summary.under_control(), 0);
        assert_eq!(*summary.area_ha(), 0.0);
        assert!(
            summary.area_ha().is_sign_positive(),
            "an empty total must print as 0.0, not -0.0"
        );
        assert!(summary.zones().is_empty());
    }
}
