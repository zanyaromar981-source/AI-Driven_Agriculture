use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration, NaiveDate, Utc};
use getset::Getters;

use crate::{
    features::plans::domain::{DailyValues, PlanAlert, PlanDecision, PlanError, PlanSource},
    shared::DomainError,
};

/// No forecast beyond ten days, ever.
pub const MAX_DAYS: usize = 10;

/// One alert of every type on every day is the most a plan could mean.
const MAX_ALERTS: usize = 100;

/// A plan whose first day is further back than this is not shown at all:
/// its sentences name weekdays that are over, and a forecast that old is
/// no longer worth a decision.
const STALE_AFTER_DAYS: i64 = 2;

/// A job's clock may run a little ahead of the server's.
const CLOCK_SLACK_MINUTES: i64 = 5;

/// The days of a plan are days in Iraq, which keeps UTC+3 all year.
const PLAN_UTC_OFFSET_HOURS: i64 = 3;

/// The day it is for a farmer at `now`.
pub fn plan_day(now: DateTime<Utc>) -> NaiveDate {
    (now + Duration::hours(PLAN_UTC_OFFSET_HOURS)).date_naive()
}

/// The current plan of one farm: the next days of weather and the farm work
/// that follows from them. A farm has at most one; the next push replaces it.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct FarmPlan {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    farm_id: i32,
    /// The day the first entry of the daily values is for.
    from: NaiveDate,
    /// When the forecast behind the plan was fetched.
    issued: DateTime<Utc>,
    daily: DailyValues,
    alerts: Vec<PlanAlert>,
    decisions: Vec<PlanDecision>,
    source: PlanSource,
    updated_at: DateTime<Utc>,
}

impl FarmPlan {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        farm_id: i32,
        from: NaiveDate,
        issued: DateTime<Utc>,
        daily: DailyValues,
        alerts: Vec<PlanAlert>,
        decisions: Vec<PlanDecision>,
        source: PlanSource,
        now: DateTime<Utc>,
    ) -> Result<Self, PlanError> {
        if farm_id < 1 {
            return Err(
                DomainError::InvalidValue("Farm id must be a positive number".to_string()).into(),
            );
        }

        // A newer `issued` always wins the upsert, so one from a clock set
        // wrong would shut every later plan out.
        if issued > now + Duration::minutes(CLOCK_SLACK_MINUTES) {
            return Err(PlanError::IssuedInTheFuture);
        }

        // Staleness is judged by `from`, so it has to be the day the plan
        // was really made. A day of slack covers the time zone.
        if (from - issued.date_naive()).num_days().abs() > 1 {
            return Err(PlanError::StartsFarFromIssue);
        }

        if alerts.len() > MAX_ALERTS {
            return Err(PlanError::TooManyAlerts { max: MAX_ALERTS });
        }

        let last = from + Duration::days(daily.days() as i64 - 1);

        if let Some(alert) = alerts
            .iter()
            .find(|alert| *alert.day() < from || *alert.day() > last)
        {
            return Err(PlanError::AlertOutsidePlan(*alert.day()));
        }

        let mut seen = HashSet::new();

        for decision in &decisions {
            if !seen.insert(*decision.code()) {
                return Err(PlanError::DuplicateDecision((*decision.code()).into()));
            }
        }

        Ok(Self {
            id: None,
            farm_id,
            from,
            issued,
            daily,
            alerts,
            decisions,
            source,
            updated_at: now,
        })
    }

    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        farm_id: i32,
        from: NaiveDate,
        issued: DateTime<Utc>,
        daily: DailyValues,
        alerts: Vec<PlanAlert>,
        decisions: Vec<PlanDecision>,
        source: PlanSource,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            farm_id,
            from,
            issued,
            daily,
            alerts,
            decisions,
            source,
            updated_at,
        }
    }

    /// Whether the plan is too old to show at `now`.
    pub fn is_stale(&self, now: DateTime<Utc>) -> bool {
        (plan_day(now) - self.from).num_days() > STALE_AFTER_DAYS
    }

    /// The plan as a farmer may see it at `now`: days that are already over
    /// are dropped with their alerts, so the first day is never in the past.
    /// A stale plan, or one with no day left, is not shown at all.
    pub fn seen_at(&self, now: DateTime<Utc>) -> Result<Self, PlanError> {
        if self.is_stale(now) {
            return Err(PlanError::Stale);
        }

        let today = plan_day(now);
        let past_days = (today - self.from).num_days();

        if past_days <= 0 {
            return Ok(self.clone());
        }

        let daily = self
            .daily
            .without_first(past_days as usize)
            .ok_or(PlanError::Stale)?;

        Ok(Self {
            from: today,
            daily,
            alerts: self
                .alerts
                .iter()
                .filter(|alert| *alert.day() >= today)
                .cloned()
                .collect(),
            ..self.clone()
        })
    }
}

/// Where one farm is, as the farms feature reports it. It has no owner on
/// purpose: the data job never learns whose farm it is.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct PlanSite {
    farm_id: i32,
    lat: f64,
    lon: f64,
}

impl PlanSite {
    /// Reconstruct from the farms feature's state.
    pub fn rehydrate(farm_id: i32, lat: f64, lon: f64) -> Self {
        Self { farm_id, lat, lon }
    }
}

/// Which farm has a plan, and when it was issued.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct PlanStamp {
    farm_id: i32,
    issued: DateTime<Utc>,
}

impl PlanStamp {
    /// Reconstruct from persisted state.
    pub fn rehydrate(farm_id: i32, issued: DateTime<Utc>) -> Self {
        Self { farm_id, issued }
    }
}

/// One farm with the time its plan was issued, so the data job can tell
/// which farms need a new one.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct PlanCoverage {
    site: PlanSite,
    /// None = the farm has no plan yet.
    issued: Option<DateTime<Utc>>,
}

impl PlanCoverage {
    /// Pairs every farm with its own plan and keeps the farms in the order
    /// they came. A plan whose farm is not listed, for example one left
    /// behind by a deleted farm, is ignored.
    pub fn assemble(sites: Vec<PlanSite>, stamps: Vec<PlanStamp>) -> Vec<Self> {
        let issued: HashMap<i32, DateTime<Utc>> = stamps
            .into_iter()
            .map(|stamp| (stamp.farm_id, stamp.issued))
            .collect();

        sites
            .into_iter()
            .map(|site| Self {
                issued: issued.get(&site.farm_id).copied(),
                site,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::features::plans::domain::{AlertLevel, AlertType, DecisionCode, PlanText};

    fn day(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).expect("date")
    }

    /// `hour` o'clock UTC on a day of October 2026.
    fn at(day: u32, hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, day, hour, 0, 0)
            .single()
            .expect("time")
    }

    fn text(value: &str) -> PlanText {
        PlanText::new(value.to_string()).expect("text")
    }

    fn daily(days: usize) -> DailyValues {
        let numbers: Vec<Option<f64>> = (0..days).map(|day| Some(day as f64)).collect();

        DailyValues::new(numbers.clone(), numbers.clone(), numbers, None).expect("values")
    }

    fn alert(on: u32) -> PlanAlert {
        PlanAlert::new(
            AlertType::Frost,
            day(on),
            Some(-3.0),
            AlertLevel::Alarm,
            text("Frost"),
            text("Frost"),
        )
        .expect("alert")
    }

    fn decision(code: DecisionCode) -> PlanDecision {
        PlanDecision::new(code, text("Wait"), text("Wait"))
    }

    fn plan(alerts: Vec<PlanAlert>, decisions: Vec<PlanDecision>) -> Result<FarmPlan, PlanError> {
        FarmPlan::new(
            7,
            day(8),
            at(8, 6),
            daily(10),
            alerts,
            decisions,
            PlanSource::new("Open-Meteo".to_string()).expect("source"),
            at(8, 6),
        )
    }

    #[test]
    fn a_plan_of_ten_days_is_valid_and_not_yet_stored() {
        let plan = plan(vec![alert(11)], vec![decision(DecisionCode::FrostCheck)]).expect("plan");

        assert!(plan.id().is_none());
        assert_eq!(plan.daily().days(), 10);
    }

    #[test]
    fn an_alert_on_the_first_or_the_last_day_is_inside_the_plan() {
        assert!(plan(vec![alert(8), alert(17)], vec![]).is_ok());
    }

    #[test]
    fn an_alert_on_a_day_the_plan_does_not_cover_is_refused() {
        assert!(
            matches!(
                plan(vec![alert(18)], vec![]),
                Err(PlanError::AlertOutsidePlan(on)) if on == day(18)
            ),
            "the app could never put its dot on a day chip"
        );
        assert!(matches!(
            plan(vec![alert(7)], vec![]),
            Err(PlanError::AlertOutsidePlan(_))
        ));
    }

    #[test]
    fn the_same_decision_twice_is_refused() {
        let result = plan(
            vec![],
            vec![
                decision(DecisionCode::SowWait),
                decision(DecisionCode::SprayOk),
                decision(DecisionCode::SowWait),
            ],
        );

        assert!(matches!(result, Err(PlanError::DuplicateDecision(code)) if code == "sow_wait"));
    }

    #[test]
    fn more_alerts_than_types_times_days_is_refused() {
        let alerts = (0..=MAX_ALERTS).map(|_| alert(9)).collect();

        assert!(matches!(
            plan(alerts, vec![]),
            Err(PlanError::TooManyAlerts { max: 100 })
        ));
    }

    #[test]
    fn a_plan_issued_in_the_future_is_refused_so_it_cannot_block_later_ones() {
        let build = |issued: DateTime<Utc>| {
            FarmPlan::new(
                7,
                day(8),
                issued,
                daily(10),
                vec![],
                vec![],
                PlanSource::new("Open-Meteo".to_string()).expect("source"),
                at(8, 6),
            )
        };

        assert!(build(at(8, 6) + Duration::minutes(4)).is_ok());
        assert!(matches!(build(at(8, 7)), Err(PlanError::IssuedInTheFuture)));
    }

    #[test]
    fn a_plan_starts_on_the_day_it_was_issued_give_or_take_a_day() {
        let build = |from: u32| {
            FarmPlan::new(
                7,
                day(from),
                at(8, 22),
                daily(10),
                vec![],
                vec![],
                PlanSource::new("Open-Meteo".to_string()).expect("source"),
                at(8, 22),
            )
        };

        assert!(
            build(9).is_ok(),
            "22:00 UTC is already the next day in Iraq"
        );
        assert!(build(7).is_ok());
        assert!(matches!(build(5), Err(PlanError::StartsFarFromIssue)));
        assert!(matches!(build(11), Err(PlanError::StartsFarFromIssue)));
    }

    #[test]
    fn a_farm_id_below_one_cannot_name_a_farm() {
        let build = |farm_id: i32| {
            FarmPlan::new(
                farm_id,
                day(8),
                at(8, 6),
                daily(10),
                vec![],
                vec![],
                PlanSource::new("Open-Meteo".to_string()).expect("source"),
                at(8, 6),
            )
        };

        assert!(build(0).is_err());
        assert!(build(-4).is_err());
    }

    #[test]
    fn the_plan_day_is_the_day_in_iraq_not_in_utc() {
        assert_eq!(plan_day(at(8, 20)), day(8));
        assert_eq!(plan_day(at(8, 21)), day(9), "21:00 UTC is midnight in Iraq");
    }

    #[test]
    fn a_plan_seen_on_its_first_day_is_shown_whole() {
        let plan = plan(vec![alert(8), alert(11)], vec![]).expect("plan");

        assert_eq!(plan.seen_at(at(8, 12)).expect("seen"), plan);
    }

    #[test]
    fn days_that_are_over_are_dropped_with_their_alerts() {
        let plan = plan(
            vec![alert(8), alert(9), alert(11)],
            vec![decision(DecisionCode::FrostCheck)],
        )
        .expect("plan");

        let seen = plan.seen_at(at(9, 12)).expect("seen");

        assert_eq!(*seen.from(), day(9), "the first day chip is today");
        assert_eq!(seen.daily().days(), 9);
        assert_eq!(seen.daily().rain_mm()[0], Some(1.0));
        assert_eq!(
            seen.alerts().iter().map(|a| *a.day()).collect::<Vec<_>>(),
            vec![day(9), day(11)]
        );
        assert_eq!(seen.decisions().len(), 1);
        assert_eq!(seen.issued(), plan.issued(), "it is still the old forecast");
    }

    #[test]
    fn a_plan_two_days_old_is_still_shown_from_today() {
        let plan = plan(vec![], vec![]).expect("plan");

        let seen = plan.seen_at(at(10, 12)).expect("seen");

        assert!(!plan.is_stale(at(10, 12)));
        assert_eq!(*seen.from(), day(10));
        assert_eq!(seen.daily().days(), 8);
    }

    #[test]
    fn a_plan_more_than_two_days_old_is_stale_and_not_shown() {
        let plan = plan(vec![], vec![]).expect("plan");

        assert!(plan.is_stale(at(11, 12)));
        assert!(
            matches!(plan.seen_at(at(11, 12)), Err(PlanError::Stale)),
            "past days must never be served as if they were the future"
        );
    }

    #[test]
    fn a_short_plan_with_no_day_left_is_not_shown() {
        let plan = FarmPlan::new(
            7,
            day(8),
            at(8, 6),
            daily(1),
            vec![],
            vec![],
            PlanSource::new("Open-Meteo".to_string()).expect("source"),
            at(8, 6),
        )
        .expect("plan");

        assert!(matches!(plan.seen_at(at(9, 12)), Err(PlanError::Stale)));
    }

    #[test]
    fn a_plan_that_starts_tomorrow_is_shown_as_it_is() {
        let plan = plan(vec![], vec![]).expect("plan");

        assert_eq!(plan.seen_at(at(7, 12)).expect("seen"), plan);
    }

    #[test]
    fn coverage_gives_each_farm_the_time_of_its_own_plan() {
        let coverage = PlanCoverage::assemble(
            vec![
                PlanSite::rehydrate(1, 36.0, 44.0),
                PlanSite::rehydrate(2, 36.5, 44.5),
            ],
            vec![PlanStamp::rehydrate(2, at(8, 6))],
        );

        assert_eq!(coverage.len(), 2);
        assert!(
            coverage[0].issued().is_none(),
            "a new farm is exactly the one the job has to plan for"
        );
        assert_eq!(*coverage[1].issued(), Some(at(8, 6)));
    }

    #[test]
    fn a_plan_of_a_farm_that_is_gone_is_left_out() {
        let coverage = PlanCoverage::assemble(
            vec![PlanSite::rehydrate(1, 36.0, 44.0)],
            vec![PlanStamp::rehydrate(99, at(8, 6))],
        );

        assert_eq!(coverage.len(), 1);
        assert!(coverage[0].issued().is_none());
    }
}
