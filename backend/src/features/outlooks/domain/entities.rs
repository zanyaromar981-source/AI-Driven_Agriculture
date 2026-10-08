use chrono::{DateTime, Utc};
use getset::Getters;

use crate::features::outlooks::domain::{
    Confidence, IssueMonth, Outlook, OutlookError, Reason, RunMethod, Season, ZoneSlug,
};

/// The outlook for one zone's next growing season, as issued in one month.
/// The same zone and season get a new outlook each month as the season nears.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct ZoneOutlook {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    zone_slug: ZoneSlug,
    season: Season,
    issued: IssueMonth,
    outlook: Outlook,
    confidence: Confidence,
    reason_en: Option<Reason>,
    reason_ku: Option<Reason>,
    updated_at: DateTime<Utc>,
}

impl ZoneOutlook {
    /// Every part is a value object that has checked itself and no rule
    /// spans two of them, so this cannot fail.
    pub fn new(
        zone_slug: ZoneSlug,
        season: Season,
        issued: IssueMonth,
        outlook: Outlook,
        confidence: Confidence,
        reason_en: Option<Reason>,
        reason_ku: Option<Reason>,
    ) -> Self {
        Self {
            id: None,
            zone_slug,
            season,
            issued,
            outlook,
            confidence,
            reason_en,
            reason_ku,
            updated_at: Utc::now(),
        }
    }

    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        zone_slug: ZoneSlug,
        season: Season,
        issued: IssueMonth,
        outlook: Outlook,
        confidence: Confidence,
        reason_en: Option<Reason>,
        reason_ku: Option<Reason>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            zone_slug,
            season,
            issued,
            outlook,
            confidence,
            reason_en,
            reason_ku,
            updated_at,
        }
    }
}

/// The track record of the method behind one issue: on how many past seasons
/// it was tested and how many of them it called right.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct OutlookRun {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    season: Season,
    issued: IssueMonth,
    seasons_tested: i32,
    seasons_right: i32,
    method: RunMethod,
    updated_at: DateTime<Utc>,
}

impl OutlookRun {
    pub fn new(
        season: Season,
        issued: IssueMonth,
        seasons_tested: i32,
        seasons_right: i32,
        method: RunMethod,
    ) -> Result<Self, OutlookError> {
        // One check covers a negative count of either kind and a method
        // that claims more right answers than it had tries.
        if seasons_tested < 0 || !(0..=seasons_tested).contains(&seasons_right) {
            return Err(OutlookError::RightOutOfRange {
                tested: seasons_tested,
                right: seasons_right,
            });
        }

        Ok(Self {
            id: None,
            season,
            issued,
            seasons_tested,
            seasons_right,
            method,
            updated_at: Utc::now(),
        })
    }

    /// Reconstruct from persisted state.
    pub fn rehydrate(
        id: i32,
        season: Season,
        issued: IssueMonth,
        seasons_tested: i32,
        seasons_right: i32,
        method: RunMethod,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            season,
            issued,
            seasons_tested,
            seasons_right,
            method,
            updated_at,
        }
    }
}

/// How many zones have each outlook.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OutlookCounts {
    pub good: usize,
    pub normal: usize,
    pub bad: usize,
}

/// One issue of the outlook for a whole season, as the dashboard shows it.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct SeasonOutlook {
    season: Season,
    issued: IssueMonth,
    counts: OutlookCounts,
    zones: Vec<ZoneOutlook>,
    track_record: Option<OutlookRun>,
    /// Every month an outlook was issued for the season, oldest first.
    issues: Vec<IssueMonth>,
}

impl SeasonOutlook {
    /// Orders the zones for the dashboard: bad first, then normal, then
    /// good, and the most confident first inside each. Zones that are still
    /// level are ordered by slug so the list does not shuffle between calls.
    pub fn new(
        season: Season,
        issued: IssueMonth,
        mut zones: Vec<ZoneOutlook>,
        track_record: Option<OutlookRun>,
        mut issues: Vec<IssueMonth>,
    ) -> Self {
        zones.sort_by(|first, second| {
            first
                .outlook
                .attention_order()
                .cmp(&second.outlook.attention_order())
                .then(
                    second
                        .confidence
                        .value()
                        .total_cmp(&first.confidence.value()),
                )
                .then(first.zone_slug.cmp(&second.zone_slug))
        });

        issues.sort();
        issues.dedup();

        let mut counts = OutlookCounts::default();

        for zone in &zones {
            match zone.outlook {
                Outlook::Good => counts.good += 1,
                Outlook::Normal => counts.normal += 1,
                Outlook::Bad => counts.bad += 1,
            }
        }

        Self {
            season,
            issued,
            counts,
            zones,
            track_record,
            issues,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn season() -> Season {
        Season::new("2026-27".to_string()).expect("season")
    }

    fn month(value: &str) -> IssueMonth {
        IssueMonth::new(value).expect("month")
    }

    fn zone(slug: &str, outlook: Outlook, confidence: f64) -> ZoneOutlook {
        ZoneOutlook::new(
            ZoneSlug::new(slug.to_string()).expect("slug"),
            season(),
            month("2026-10"),
            outlook,
            Confidence::new(confidence).expect("confidence"),
            None,
            None,
        )
    }

    fn run(tested: i32, right: i32) -> Result<OutlookRun, OutlookError> {
        OutlookRun::new(
            season(),
            month("2026-10"),
            tested,
            right,
            RunMethod::new("analog years".to_string()).expect("method"),
        )
    }

    fn slugs(board: &SeasonOutlook) -> Vec<&str> {
        board
            .zones()
            .iter()
            .map(|zone| zone.zone_slug().as_str())
            .collect()
    }

    #[test]
    fn a_new_zone_outlook_is_not_yet_persisted() {
        let outlook = zone("makhmur", Outlook::Bad, 80.0);

        assert_eq!(*outlook.id(), None);
        assert_eq!(*outlook.outlook(), Outlook::Bad);
    }

    #[test]
    fn a_method_right_in_some_of_its_tested_seasons_is_a_track_record() {
        let record = run(25, 19).expect("run");

        assert_eq!(*record.seasons_tested(), 25);
        assert_eq!(*record.seasons_right(), 19);
    }

    #[test]
    fn right_in_all_or_in_none_of_the_seasons_is_allowed() {
        assert!(run(25, 25).is_ok());
        assert!(run(25, 0).is_ok());
        assert!(run(0, 0).is_ok(), "a method not tested yet has no record");
    }

    #[test]
    fn a_method_cannot_be_right_more_often_than_it_was_tested() {
        assert!(matches!(
            run(25, 26),
            Err(OutlookError::RightOutOfRange {
                tested: 25,
                right: 26
            })
        ));
    }

    #[test]
    fn negative_counts_are_refused() {
        assert!(run(-1, 0).is_err());
        assert!(run(25, -1).is_err());
        assert!(run(-5, -5).is_err());
    }

    #[test]
    fn bad_zones_lead_then_normal_then_good() {
        let board = SeasonOutlook::new(
            season(),
            month("2026-10"),
            vec![
                zone("erbil", Outlook::Good, 90.0),
                zone("makhmur", Outlook::Bad, 60.0),
                zone("koya", Outlook::Normal, 99.0),
            ],
            None,
            vec![month("2026-10")],
        );

        assert_eq!(
            slugs(&board),
            vec!["makhmur", "koya", "erbil"],
            "the outlook decides before the confidence does"
        );
    }

    #[test]
    fn the_most_confident_comes_first_inside_one_outlook() {
        let board = SeasonOutlook::new(
            season(),
            month("2026-10"),
            vec![
                zone("chamchamal", Outlook::Bad, 55.0),
                zone("makhmur", Outlook::Bad, 85.5),
                zone("kalar", Outlook::Bad, 70.0),
            ],
            None,
            vec![month("2026-10")],
        );

        assert_eq!(slugs(&board), vec!["makhmur", "kalar", "chamchamal"]);
    }

    #[test]
    fn zones_level_on_both_are_ordered_by_slug() {
        let board = SeasonOutlook::new(
            season(),
            month("2026-10"),
            vec![
                zone("makhmur", Outlook::Bad, 70.0),
                zone("chamchamal", Outlook::Bad, 70.0),
            ],
            None,
            vec![month("2026-10")],
        );

        assert_eq!(slugs(&board), vec!["chamchamal", "makhmur"]);
    }

    #[test]
    fn the_counts_cover_every_zone_once() {
        let board = SeasonOutlook::new(
            season(),
            month("2026-10"),
            vec![
                zone("erbil", Outlook::Good, 90.0),
                zone("makhmur", Outlook::Bad, 60.0),
                zone("kalar", Outlook::Bad, 60.0),
                zone("koya", Outlook::Normal, 50.0),
            ],
            None,
            vec![month("2026-10")],
        );

        assert_eq!(
            *board.counts(),
            OutlookCounts {
                good: 1,
                normal: 1,
                bad: 2
            }
        );
    }

    #[test]
    fn an_issue_without_zones_counts_nothing() {
        let board = SeasonOutlook::new(season(), month("2026-10"), vec![], None, vec![]);

        assert_eq!(*board.counts(), OutlookCounts::default());
        assert!(board.zones().is_empty());
    }

    #[test]
    fn the_issue_months_are_listed_oldest_first_and_once() {
        let board = SeasonOutlook::new(
            season(),
            month("2026-10"),
            vec![],
            None,
            vec![
                month("2026-10"),
                month("2026-08"),
                month("2026-09"),
                month("2026-08"),
            ],
        );

        assert_eq!(
            *board.issues(),
            vec![month("2026-08"), month("2026-09"), month("2026-10")]
        );
    }

    #[test]
    fn the_track_record_travels_with_the_issue() {
        let board = SeasonOutlook::new(
            season(),
            month("2026-10"),
            vec![],
            Some(run(25, 19).expect("run")),
            vec![month("2026-10")],
        );

        assert_eq!(
            board
                .track_record()
                .as_ref()
                .map(|record| (*record.seasons_tested(), *record.seasons_right())),
            Some((25, 19))
        );
    }
}
