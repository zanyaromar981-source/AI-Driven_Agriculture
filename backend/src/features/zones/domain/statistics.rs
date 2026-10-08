//! The arithmetic behind the dashboard numbers: ranking, means and rounding.
//! There are 33 zones, so everything here works on small slices in memory.

use std::collections::HashMap;

use crate::features::zones::domain::Dryness;

/// Ranks zones by dryness: rank 1 is the driest. Zones with the same dryness
/// share a rank and the ranks after them are skipped (1, 2, 2, 4), so a rank
/// always says how many zones are drier, plus one.
pub(super) fn ranks(dryness_by_zone: &[(i32, Dryness)]) -> HashMap<i32, u32> {
    let mut driest_first = dryness_by_zone.to_vec();
    driest_first.sort_by(|(_, first), (_, second)| second.cmp(first));

    let mut ranks = HashMap::new();
    let mut previous: Option<(Dryness, u32)> = None;

    for (position, (zone_id, dryness)) in driest_first.into_iter().enumerate() {
        let rank = match previous {
            Some((same, rank)) if same == dryness => rank,
            _ => u32::try_from(position + 1).unwrap_or(u32::MAX),
        };

        previous = Some((dryness, rank));
        ranks.insert(zone_id, rank);
    }

    ranks
}

/// The plain mean, not rounded. `None` when there is nothing to average,
/// which is not the same as an average of zero.
pub(super) fn mean(values: &[Dryness]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    let sum: i32 = values.iter().map(Dryness::value).sum();

    Some(f64::from(sum) / values.len() as f64)
}

/// Averages are shown with one decimal.
pub(super) fn round_to_one_decimal(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dryness(value: i32) -> Dryness {
        Dryness::new(value).expect("dryness")
    }

    #[test]
    fn the_driest_zone_is_rank_one() {
        let ranks = ranks(&[(1, dryness(40)), (2, dryness(90)), (3, dryness(65))]);

        assert_eq!(ranks[&2], 1);
        assert_eq!(ranks[&3], 2);
        assert_eq!(ranks[&1], 3);
    }

    #[test]
    fn zones_equally_dry_share_a_rank_and_the_next_one_is_skipped() {
        let ranks = ranks(&[
            (1, dryness(80)),
            (2, dryness(70)),
            (3, dryness(70)),
            (4, dryness(10)),
        ]);

        assert_eq!(ranks[&1], 1);
        assert_eq!(ranks[&2], 2);
        assert_eq!(ranks[&3], 2);
        assert_eq!(
            ranks[&4], 4,
            "three zones are drier, so this one is fourth, not third"
        );
    }

    #[test]
    fn nothing_to_rank_gives_no_ranks() {
        assert!(ranks(&[]).is_empty());
    }

    #[test]
    fn the_mean_of_nothing_is_missing_not_zero() {
        assert_eq!(mean(&[]), None);
    }

    #[test]
    fn the_mean_is_not_rounded_until_it_is_shown() {
        let mean = mean(&[dryness(10), dryness(20), dryness(25)]).expect("mean");

        assert!((mean - 18.333_333).abs() < 1e-5);
        assert_eq!(round_to_one_decimal(mean), 18.3);
    }

    #[test]
    fn rounding_keeps_one_decimal_and_the_sign() {
        assert_eq!(round_to_one_decimal(62.25), 62.3);
        assert_eq!(round_to_one_decimal(-4.44), -4.4);
        assert_eq!(round_to_one_decimal(50.0), 50.0);
    }
}
