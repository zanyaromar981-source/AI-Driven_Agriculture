use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, Utc};
use getset::Getters;

use crate::{
    features::alerts::domain::{
        AlertConfidence, AlertError, AlertKey, AlertLevel, AlertSource, AlertText, AlertType,
        DeviceLanguage, Platform, PushToken,
    },
    shared::{DomainError, Phone},
};

/// Iraq keeps UTC+3 all year. "Today" in the push rule is the farmer's day,
/// not the server's.
const BAGHDAD_OFFSET_HOURS: i64 = 3;

/// An alarm for a day further back than this is old news and is not pushed.
const PUSH_GRACE_DAYS: i64 = 1;

/// The day it is in Iraq at `now`.
pub fn baghdad_day(now: DateTime<Utc>) -> NaiveDate {
    (now + Duration::hours(BAGHDAD_OFFSET_HOURS)).date_naive()
}

/// The moment the Iraqi day of `now` began.
pub fn baghdad_day_start(now: DateTime<Utc>) -> DateTime<Utc> {
    baghdad_day(now).and_time(NaiveTime::MIN).and_utc() - Duration::hours(BAGHDAD_OFFSET_HOURS)
}

/// One thing a farmer should know about one farm on one day, with what to
/// do about it. A job names it with a key and may send it again and again.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Alert {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    farm_id: i32,
    key: AlertKey,
    alert_type: AlertType,
    /// The day the alert is about, which may be days ahead.
    day: NaiveDate,
    level: AlertLevel,
    confidence: AlertConfidence,
    ku: AlertText,
    en: AlertText,
    action_ku: AlertText,
    action_en: AlertText,
    pushed: bool,
    pushed_at: Option<DateTime<Utc>>,
    /// Ticked by the farmer. A job sending the alert again never unticks it.
    done: bool,
    done_at: Option<DateTime<Utc>>,
    source: AlertSource,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Alert {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        farm_id: i32,
        key: AlertKey,
        alert_type: AlertType,
        day: NaiveDate,
        level: AlertLevel,
        confidence: AlertConfidence,
        ku: AlertText,
        en: AlertText,
        action_ku: AlertText,
        action_en: AlertText,
        source: AlertSource,
        now: DateTime<Utc>,
    ) -> Result<Self, AlertError> {
        if farm_id < 1 {
            return Err(
                DomainError::InvalidValue("Farm id must be a positive number".to_string()).into(),
            );
        }

        Ok(Self {
            id: None,
            farm_id,
            key,
            alert_type,
            day,
            level,
            confidence,
            ku,
            en,
            action_ku,
            action_en,
            pushed: false,
            pushed_at: None,
            done: false,
            done_at: None,
            source,
            created_at: now,
            updated_at: now,
        })
    }

    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        farm_id: i32,
        key: AlertKey,
        alert_type: AlertType,
        day: NaiveDate,
        level: AlertLevel,
        confidence: AlertConfidence,
        ku: AlertText,
        en: AlertText,
        action_ku: AlertText,
        action_en: AlertText,
        pushed: bool,
        pushed_at: Option<DateTime<Utc>>,
        done: bool,
        done_at: Option<DateTime<Utc>>,
        source: AlertSource,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            farm_id,
            key,
            alert_type,
            day,
            level,
            confidence,
            ku,
            en,
            action_ku,
            action_en,
            pushed,
            pushed_at,
            done,
            done_at,
            source,
            created_at,
            updated_at,
        }
    }

    /// Whether the alert may still go to a phone: only an alarm, only once,
    /// not after the farmer ticked it, and not for a day long past.
    pub fn is_due_for_push(&self, today: NaiveDate) -> bool {
        self.level == AlertLevel::Alarm
            && !self.pushed
            && !self.done
            && self.day >= today - Duration::days(PUSH_GRACE_DAYS)
    }
}

/// Applies "at most one push per farm per day": of the alerts that are due,
/// keeps one per farm and none for a farm that already had a push today.
/// The one kept is the alert for the nearest day, then the oldest. Ordered
/// by farm.
pub fn one_per_farm(
    alerts: Vec<Alert>,
    farms_pushed_today: &HashSet<i32>,
    today: NaiveDate,
) -> Vec<Alert> {
    let mut chosen: BTreeMap<i32, Alert> = BTreeMap::new();

    for alert in alerts {
        if !alert.is_due_for_push(today) || farms_pushed_today.contains(&alert.farm_id) {
            continue;
        }

        let better = match chosen.get(&alert.farm_id) {
            Some(kept) => (alert.day, alert.id) < (kept.day, kept.id),
            None => true,
        };

        if better {
            chosen.insert(alert.farm_id, alert);
        }
    }

    chosen.into_values().collect()
}

/// An alarm that may be pushed now, with the phones to push it to.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct PendingPush {
    alert: Alert,
    /// The owner's phones that want red alerts. Empty when there is none.
    devices: Vec<Device>,
}

impl PendingPush {
    pub fn new(alert: Alert, devices: Vec<Device>) -> Self {
        Self { alert, devices }
    }
}

/// One phone that may receive pushes, and what its owner wants pushed.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Device {
    push_token: PushToken,
    /// The farmer the phone belongs to now. A token that signs in under
    /// another number moves to that number.
    phone: Phone,
    platform: Platform,
    lang: DeviceLanguage,
    red_alerts: bool,
    weekly_plan: bool,
    updated_at: DateTime<Utc>,
}

impl Device {
    pub fn new(
        push_token: PushToken,
        phone: Phone,
        platform: Platform,
        lang: DeviceLanguage,
        red_alerts: bool,
        weekly_plan: bool,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            push_token,
            phone,
            platform,
            lang,
            red_alerts,
            weekly_plan,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(day_of_month: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day_of_month).expect("day")
    }

    fn text(value: &str) -> AlertText {
        AlertText::new(value.to_string()).expect("text")
    }

    fn alert(id: i32, farm_id: i32, on: NaiveDate, level: AlertLevel) -> Alert {
        Alert::rehydrate(
            id,
            farm_id,
            AlertKey::new(format!("frost:{on}:{id}")).expect("key"),
            AlertType::Frost,
            on,
            level,
            AlertConfidence::Likely,
            text("ku"),
            text("Frost"),
            text("ku"),
            text("Cover the seedlings"),
            false,
            None,
            false,
            None,
            AlertSource::new("test".to_string()).expect("source"),
            Utc::now(),
            Utc::now(),
        )
    }

    fn with_flags(alert: Alert, pushed: bool, done: bool) -> Alert {
        Alert {
            pushed,
            done,
            ..alert
        }
    }

    #[test]
    fn a_new_alert_is_neither_pushed_nor_done() {
        let now = Utc::now();
        let alert = Alert::new(
            7,
            AlertKey::new("frost:2026-10-11".to_string()).expect("key"),
            AlertType::Frost,
            day(11),
            AlertLevel::Alarm,
            AlertConfidence::Likely,
            text("ku"),
            text("Frost"),
            text("ku"),
            text("Cover"),
            AlertSource::new("test".to_string()).expect("source"),
            now,
        )
        .expect("alert");

        assert!(alert.id().is_none());
        assert!(!alert.pushed() && !alert.done());
        assert!(alert.pushed_at().is_none() && alert.done_at().is_none());
        assert_eq!(*alert.created_at(), now);
    }

    #[test]
    fn an_alert_needs_a_real_farm_id() {
        let result = Alert::new(
            0,
            AlertKey::new("k".to_string()).expect("key"),
            AlertType::Fire,
            day(9),
            AlertLevel::Watch,
            AlertConfidence::Unsure,
            text("a"),
            text("a"),
            text("a"),
            text("a"),
            AlertSource::new("test".to_string()).expect("source"),
            Utc::now(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn the_iraqi_day_starts_three_hours_before_the_utc_day() {
        let late = "2026-10-09T21:30:00Z"
            .parse::<DateTime<Utc>>()
            .expect("time");

        assert_eq!(baghdad_day(late), day(10), "21:30 UTC is 00:30 in Iraq");
        assert_eq!(
            baghdad_day_start(late),
            "2026-10-09T21:00:00Z"
                .parse::<DateTime<Utc>>()
                .expect("time")
        );

        let early = "2026-10-09T20:59:00Z"
            .parse::<DateTime<Utc>>()
            .expect("time");

        assert_eq!(baghdad_day(early), day(9));
    }

    #[test]
    fn only_an_alarm_is_due_for_a_push_never_a_watch() {
        assert!(alert(1, 7, day(9), AlertLevel::Alarm).is_due_for_push(day(9)));
        assert!(!alert(1, 7, day(9), AlertLevel::Watch).is_due_for_push(day(9)));
    }

    #[test]
    fn a_pushed_or_ticked_alarm_is_not_pushed_again() {
        let alarm = alert(1, 7, day(9), AlertLevel::Alarm);

        assert!(!with_flags(alarm.clone(), true, false).is_due_for_push(day(9)));
        assert!(
            !with_flags(alarm, false, true).is_due_for_push(day(9)),
            "the farmer has already dealt with it"
        );
    }

    #[test]
    fn an_alarm_for_a_day_long_past_is_not_pushed() {
        assert!(alert(1, 7, day(8), AlertLevel::Alarm).is_due_for_push(day(9)));
        assert!(!alert(1, 7, day(7), AlertLevel::Alarm).is_due_for_push(day(9)));
        assert!(alert(1, 7, day(15), AlertLevel::Alarm).is_due_for_push(day(9)));
    }

    #[test]
    fn a_farm_with_two_alarms_gets_only_the_one_for_the_nearest_day() {
        let chosen = one_per_farm(
            vec![
                alert(1, 7, day(12), AlertLevel::Alarm),
                alert(2, 7, day(10), AlertLevel::Alarm),
                alert(3, 8, day(11), AlertLevel::Alarm),
            ],
            &HashSet::new(),
            day(9),
        );

        let ids: Vec<i32> = chosen.iter().map(|alert| alert.id().expect("id")).collect();

        assert_eq!(ids, vec![2, 3]);
    }

    #[test]
    fn a_farm_that_already_had_a_push_today_is_left_out() {
        let chosen = one_per_farm(
            vec![
                alert(1, 7, day(9), AlertLevel::Alarm),
                alert(2, 8, day(9), AlertLevel::Alarm),
            ],
            &HashSet::from([7]),
            day(9),
        );

        assert_eq!(chosen.len(), 1);
        assert_eq!(*chosen[0].farm_id(), 8);
    }

    #[test]
    fn two_alarms_for_the_same_day_give_the_older_one() {
        let chosen = one_per_farm(
            vec![
                alert(5, 7, day(9), AlertLevel::Alarm),
                alert(4, 7, day(9), AlertLevel::Alarm),
            ],
            &HashSet::new(),
            day(9),
        );

        assert_eq!(*chosen[0].id(), Some(4));
    }
}
