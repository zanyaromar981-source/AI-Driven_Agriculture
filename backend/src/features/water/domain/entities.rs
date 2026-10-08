use chrono::{DateTime, Utc};
use getset::Getters;

use crate::{
    features::water::domain::{DamSlug, Need, Note, Season, WaterError, ZoneSlug},
    shared::DomainError,
};

/// One zone's line in a season's water plan: how much it needs water and,
/// once decided, how much it is to get and from which dam.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct WaterPlanEntry {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    season: Season,
    zone_slug: ZoneSlug,
    need: Need,
    dam_slug: Option<DamSlug>,
    send_million_m3: Option<f64>,
    urgent: bool,
    note_en: Option<Note>,
    note_ku: Option<Note>,
    updated_at: DateTime<Utc>,
}

impl WaterPlanEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        season: Season,
        zone_slug: ZoneSlug,
        need: Need,
        dam_slug: Option<DamSlug>,
        send_million_m3: Option<f64>,
        urgent: bool,
        note_en: Option<Note>,
        note_ku: Option<Note>,
    ) -> Result<Self, WaterError> {
        // A NaN is not below zero, so it needs its own check.
        if let Some(send) = send_million_m3
            && (!send.is_finite() || send < 0.0)
        {
            return Err(DomainError::InvalidValue(
                "send_million_m3 must not be negative".to_string(),
            )
            .into());
        }

        Ok(Self {
            id: None,
            season,
            zone_slug,
            need,
            dam_slug,
            send_million_m3,
            urgent,
            note_en,
            note_ku,
            updated_at: Utc::now(),
        })
    }

    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        season: Season,
        zone_slug: ZoneSlug,
        need: Need,
        dam_slug: Option<DamSlug>,
        send_million_m3: Option<f64>,
        urgent: bool,
        note_en: Option<Note>,
        note_ku: Option<Note>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            season,
            zone_slug,
            need,
            dam_slug,
            send_million_m3,
            urgent,
            note_en,
            note_ku,
            updated_at,
        }
    }
}

/// An entry with its place in the plan. Rank 1 is the zone that needs water
/// most.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct RankedEntry {
    rank: usize,
    entry: WaterPlanEntry,
}

/// What one dam is asked to give under the plan.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct DamAllocation {
    dam_slug: DamSlug,
    planned_million_m3: f64,
    /// How many zones the plan supplies from this dam.
    zones: usize,
}

#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct PlanTotals {
    /// Every volume in the plan, whether or not its dam is decided yet.
    planned_million_m3: f64,
    urgent_zones: usize,
    /// Largest volume first. Entries without a dam are not in any of these,
    /// so the dams can add up to less than the plan's total.
    by_dam: Vec<DamAllocation>,
}

/// A season's water plan as the dashboard shows it.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct WaterPlan {
    season: Season,
    entries: Vec<RankedEntry>,
    totals: PlanTotals,
}

impl WaterPlan {
    /// Ranks the entries, highest need first, and adds up the totals. Zones
    /// with the same need are ranked by slug, so the order does not shuffle
    /// between calls; each still gets its own rank.
    pub fn new(season: Season, mut entries: Vec<WaterPlanEntry>) -> Self {
        entries.sort_by(|first, second| {
            second
                .need
                .value()
                .total_cmp(&first.need.value())
                .then(first.zone_slug.cmp(&second.zone_slug))
        });

        let totals = totals(&entries);

        let entries = entries
            .into_iter()
            .zip(1..)
            .map(|(entry, rank)| RankedEntry { rank, entry })
            .collect();

        Self {
            season,
            entries,
            totals,
        }
    }
}

fn totals(entries: &[WaterPlanEntry]) -> PlanTotals {
    let mut by_dam: Vec<DamAllocation> = Vec::new();

    for entry in entries {
        let Some(dam_slug) = &entry.dam_slug else {
            continue;
        };

        let send = entry.send_million_m3.unwrap_or(0.0);

        match by_dam
            .iter_mut()
            .find(|allocation| &allocation.dam_slug == dam_slug)
        {
            Some(allocation) => {
                allocation.planned_million_m3 += send;
                allocation.zones += 1;
            }
            None => by_dam.push(DamAllocation {
                dam_slug: dam_slug.clone(),
                planned_million_m3: send,
                zones: 1,
            }),
        }
    }

    by_dam.sort_by(|first, second| {
        second
            .planned_million_m3
            .total_cmp(&first.planned_million_m3)
            .then(first.dam_slug.cmp(&second.dam_slug))
    });

    PlanTotals {
        // Folded from zero rather than summed: the sum of no floats is
        // minus zero, which would reach the dashboard as "-0.0".
        planned_million_m3: entries
            .iter()
            .filter_map(|entry| entry.send_million_m3)
            .fold(0.0, |total, send| total + send),
        urgent_zones: entries.iter().filter(|entry| entry.urgent).count(),
        by_dam,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn season() -> Season {
        Season::new("2026-27".to_string()).expect("season")
    }

    fn entry(
        zone: &str,
        need: f64,
        dam: Option<&str>,
        send: Option<f64>,
        urgent: bool,
    ) -> WaterPlanEntry {
        WaterPlanEntry::new(
            season(),
            ZoneSlug::new(zone.to_string()).expect("zone"),
            Need::new(need).expect("need"),
            dam.map(|dam| DamSlug::new(dam.to_string()).expect("dam")),
            send,
            urgent,
            None,
            None,
        )
        .expect("entry")
    }

    fn ranking(plan: &WaterPlan) -> Vec<(usize, &str)> {
        plan.entries()
            .iter()
            .map(|ranked| (*ranked.rank(), ranked.entry().zone_slug().as_str()))
            .collect()
    }

    #[test]
    fn a_new_entry_is_not_yet_persisted() {
        let entry = entry("makhmur", 90.0, Some("dukan"), Some(40.0), true);

        assert_eq!(*entry.id(), None);
        assert_eq!(*entry.send_million_m3(), Some(40.0));
    }

    #[test]
    fn an_entry_may_wait_for_its_dam_and_its_volume() {
        let entry = entry("makhmur", 90.0, None, None, false);

        assert!(entry.dam_slug().is_none());
        assert!(entry.send_million_m3().is_none());
    }

    #[test]
    fn a_negative_volume_is_refused_and_zero_is_not() {
        let with = |send: f64| {
            WaterPlanEntry::new(
                season(),
                ZoneSlug::new("makhmur".to_string()).expect("zone"),
                Need::new(50.0).expect("need"),
                None,
                Some(send),
                false,
                None,
                None,
            )
        };

        assert!(with(-0.5).is_err());
        assert!(with(f64::NAN).is_err());
        assert!(with(f64::INFINITY).is_err());
        assert!(with(0.0).is_ok());
    }

    #[test]
    fn the_zone_that_needs_water_most_is_rank_one() {
        let plan = WaterPlan::new(
            season(),
            vec![
                entry("koya", 40.0, None, None, false),
                entry("makhmur", 92.5, None, None, false),
                entry("kalar", 71.0, None, None, false),
            ],
        );

        assert_eq!(
            ranking(&plan),
            vec![(1, "makhmur"), (2, "kalar"), (3, "koya")]
        );
    }

    #[test]
    fn zones_with_the_same_need_are_ranked_by_slug_each_with_its_own_rank() {
        let plan = WaterPlan::new(
            season(),
            vec![
                entry("makhmur", 80.0, None, None, false),
                entry("chamchamal", 80.0, None, None, false),
                entry("kalar", 95.0, None, None, false),
            ],
        );

        assert_eq!(
            ranking(&plan),
            vec![(1, "kalar"), (2, "chamchamal"), (3, "makhmur")]
        );
    }

    #[test]
    fn being_urgent_does_not_change_the_rank() {
        let plan = WaterPlan::new(
            season(),
            vec![
                entry("koya", 30.0, None, None, true),
                entry("makhmur", 90.0, None, None, false),
            ],
        );

        assert_eq!(
            ranking(&plan),
            vec![(1, "makhmur"), (2, "koya")],
            "the rank is the need alone: urgency is shown beside it, not folded in"
        );
    }

    #[test]
    fn the_totals_add_up_the_volumes_and_count_the_urgent_zones() {
        let plan = WaterPlan::new(
            season(),
            vec![
                entry("makhmur", 90.0, Some("dukan"), Some(40.0), true),
                entry("koya", 60.0, Some("dukan"), Some(25.5), false),
                entry("kalar", 80.0, Some("darbandikhan"), Some(30.0), true),
                entry("chamchamal", 20.0, None, None, false),
            ],
        );

        assert_eq!(*plan.totals().planned_million_m3(), 95.5);
        assert_eq!(*plan.totals().urgent_zones(), 2);
    }

    #[test]
    fn each_dam_is_listed_once_with_its_volume_and_zones_largest_first() {
        let plan = WaterPlan::new(
            season(),
            vec![
                entry("kalar", 80.0, Some("darbandikhan"), Some(30.0), false),
                entry("makhmur", 90.0, Some("dukan"), Some(40.0), false),
                entry("koya", 60.0, Some("dukan"), Some(25.5), false),
            ],
        );

        assert_eq!(
            plan.totals()
                .by_dam()
                .iter()
                .map(|dam| (
                    dam.dam_slug().as_str(),
                    *dam.planned_million_m3(),
                    *dam.zones()
                ))
                .collect::<Vec<_>>(),
            vec![("dukan", 65.5, 2), ("darbandikhan", 30.0, 1)]
        );
    }

    #[test]
    fn a_zone_with_a_dam_but_no_volume_yet_counts_as_a_zone_of_that_dam() {
        let plan = WaterPlan::new(
            season(),
            vec![
                entry("makhmur", 90.0, Some("dukan"), Some(40.0), false),
                entry("koya", 60.0, Some("dukan"), None, false),
            ],
        );

        let dukan = &plan.totals().by_dam()[0];

        assert_eq!(*dukan.zones(), 2);
        assert_eq!(*dukan.planned_million_m3(), 40.0);
    }

    #[test]
    fn a_volume_without_a_dam_is_in_the_total_but_under_no_dam() {
        let plan = WaterPlan::new(
            season(),
            vec![
                entry("makhmur", 90.0, Some("dukan"), Some(40.0), false),
                entry("koya", 60.0, None, Some(10.0), false),
            ],
        );

        assert_eq!(*plan.totals().planned_million_m3(), 50.0);
        assert_eq!(plan.totals().by_dam().len(), 1);
        assert_eq!(*plan.totals().by_dam()[0].planned_million_m3(), 40.0);
    }

    #[test]
    fn an_empty_plan_has_no_entries_and_zero_totals() {
        let plan = WaterPlan::new(season(), vec![]);

        assert!(plan.entries().is_empty());
        assert_eq!(*plan.totals().planned_million_m3(), 0.0);
        assert!(
            plan.totals().planned_million_m3().is_sign_positive(),
            "nothing planned must read as 0.0, not as -0.0"
        );
        assert_eq!(*plan.totals().urgent_zones(), 0);
        assert!(plan.totals().by_dam().is_empty());
    }
}
