//! How a job's runs are judged. Everything here is a pure function of the
//! job, its runs and `now`.
//!
//! The state of a job, first match wins:
//!
//! 1. `never`: the job has no run at all.
//! 2. `failed`: the run that finished last (by `finished_at`) was not ok. A
//!    run still going does not hide the failure before it.
//! 3. `late`: the job has `every_hours`, and no run finished ok within
//!    `every_hours` x 1.5 before `now`. A job with no `every_hours` runs on
//!    demand and is never late.
//! 4. `ok`: otherwise.
//!
//! One day of `last_14_days`. Days are UTC calendar days, the last entry is
//! today, and a run belongs to the day it started on. First match wins:
//!
//! 1. no mark (null): the day is before the day of the job's first ever run
//!    (so a job that never ran has no mark at all).
//! 2. `failed`: any run that started that day finished not ok.
//! 3. `ok`: at least one run that started that day finished ok.
//! 4. `late`: the job has `every_hours` and was due with nothing to show for
//!    it. For a day that is over, that means no run finished ok in the
//!    `every_hours` before the day ended. Today is not over, so it is judged
//!    as the state is: no run finished ok within `every_hours` x 1.5 before
//!    `now`.
//! 5. no mark (null): otherwise. The job runs on demand and was not asked
//!    that day, or its period is longer than a day and it was not due yet.

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, Utc};
use getset::Getters;

use crate::features::jobs::domain::{DayMark, Job, JobRun, JobState, RunMessage};

/// How many days the dashboard shows per job, today included.
pub const DAYS_SHOWN: usize = 14;

/// A job is late once one and a half of its periods have passed with no run
/// that finished ok: one missed run plus half a period of patience.
fn patience(period: Duration) -> Duration {
    period + period / 2
}

fn start_of(day: NaiveDate) -> DateTime<Utc> {
    day.and_time(NaiveTime::MIN).and_utc()
}

/// The days shown, oldest first, ending today.
fn shown_days(now: DateTime<Utc>) -> [NaiveDate; DAYS_SHOWN] {
    let today = now.date_naive();

    std::array::from_fn(|index| today - Duration::days((DAYS_SHOWN - 1 - index) as i64))
}

/// The runs that must be loaded in full to judge `jobs` at `now`: those that
/// started at or after this moment. It reaches back before the first day
/// shown by the longest period, because whether a job was due on that day
/// depends on the runs of the period before it.
pub fn history_since(jobs: &[Job], now: DateTime<Utc>) -> DateTime<Utc> {
    let longest = jobs
        .iter()
        .filter_map(Job::period)
        .max()
        .unwrap_or_else(Duration::zero);

    start_of(shown_days(now)[0]) - longest
}

/// What is known of one job's runs: a few runs picked out of its whole
/// history, and its recent runs in full.
#[derive(Clone, Debug, Default, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct RunHistory {
    /// When the job's first ever run started.
    first_started_at: Option<DateTime<Utc>>,
    /// The run that started last.
    latest: Option<JobRun>,
    /// The run that finished last, ok or not.
    latest_finished: Option<JobRun>,
    /// The run that finished ok last.
    latest_ok: Option<JobRun>,
    /// Every run that started at or after [`history_since`].
    recent: Vec<JobRun>,
}

impl RunHistory {
    pub fn new(
        first_started_at: Option<DateTime<Utc>>,
        latest: Option<JobRun>,
        latest_finished: Option<JobRun>,
        latest_ok: Option<JobRun>,
        recent: Vec<JobRun>,
    ) -> Self {
        Self {
            first_started_at,
            latest,
            latest_finished,
            latest_ok,
            recent,
        }
    }

    /// The history of a job that never ran.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Picks the history out of all of one job's runs, in any order.
    pub fn of(runs: &[JobRun], since: DateTime<Utc>) -> Self {
        Self {
            first_started_at: runs.iter().map(|run| *run.started_at()).min(),
            latest: runs.iter().max_by_key(|run| *run.started_at()).cloned(),
            latest_finished: runs
                .iter()
                .filter(|run| run.finished_at().is_some())
                .max_by_key(|run| *run.finished_at())
                .cloned(),
            latest_ok: runs
                .iter()
                .filter(|run| run.finished_ok_at().is_some())
                .max_by_key(|run| run.finished_ok_at())
                .cloned(),
            recent: runs
                .iter()
                .filter(|run| *run.started_at() >= since)
                .cloned()
                .collect(),
        }
    }

    /// When a run last finished ok, at or before `moment`.
    fn last_ok_by(&self, moment: DateTime<Utc>) -> Option<DateTime<Utc>> {
        self.recent
            .iter()
            .chain(self.latest_ok.iter())
            .filter_map(JobRun::finished_ok_at)
            .filter(|finished_at| *finished_at <= moment)
            .max()
    }

    /// No run finished ok in the `allowed` time before `moment`.
    fn nothing_ok_within(&self, allowed: Duration, moment: DateTime<Utc>) -> bool {
        self.last_ok_by(moment)
            .is_none_or(|finished_at| moment - finished_at > allowed)
    }
}

/// One job as the dashboard shows it: what it is, how it stands now and how
/// its last days went.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct JobStatus {
    job: Job,
    /// The run that started last, also when it is still going.
    last_run: Option<JobRun>,
    /// When a run last finished ok.
    last_ok: Option<DateTime<Utc>>,
    /// One period after the last run started. None for a job on demand and
    /// for one that never ran.
    next_due: Option<DateTime<Utc>>,
    state: JobState,
    /// Oldest first, today last.
    last_14_days: [Option<DayMark>; DAYS_SHOWN],
    /// What the run that finished last said, so that a failure's reason
    /// stays on screen while the next run is going. Before any run has
    /// finished it is what the run now going said.
    message: Option<RunMessage>,
}

impl JobStatus {
    /// Judges one job at `now`. The rules are at the top of this file.
    pub fn judge(job: &Job, history: &RunHistory, now: DateTime<Utc>) -> Self {
        let last_ok = history.latest_ok.as_ref().and_then(JobRun::finished_ok_at);

        let next_due = job
            .period()
            .zip(history.latest.as_ref())
            .map(|(period, latest)| *latest.started_at() + period);

        let message = history
            .latest_finished
            .as_ref()
            .or(history.latest.as_ref())
            .and_then(|run| run.message().clone());

        Self {
            job: job.clone(),
            last_run: history.latest.clone(),
            last_ok,
            next_due,
            state: state_of(job, history, now),
            last_14_days: shown_days(now).map(|day| mark_of(job, history, day, now)),
            message,
        }
    }
}

fn state_of(job: &Job, history: &RunHistory, now: DateTime<Utc>) -> JobState {
    if history.latest.is_none() {
        return JobState::Never;
    }

    if history.latest_finished.as_ref().is_some_and(JobRun::failed) {
        return JobState::Failed;
    }

    if job
        .period()
        .is_some_and(|period| history.nothing_ok_within(patience(period), now))
    {
        return JobState::Late;
    }

    JobState::Ok
}

fn mark_of(job: &Job, history: &RunHistory, day: NaiveDate, now: DateTime<Utc>) -> Option<DayMark> {
    let first_day = history.first_started_at?.date_naive();

    if day < first_day {
        return None;
    }

    let that_day: Vec<&JobRun> = history
        .recent
        .iter()
        .filter(|run| run.started_at().date_naive() == day)
        .collect();

    if that_day.iter().any(|run| run.failed()) {
        return Some(DayMark::Failed);
    }

    if that_day.iter().any(|run| run.finished_ok_at().is_some()) {
        return Some(DayMark::Ok);
    }

    let period = job.period()?;

    let due_and_missed = if day == now.date_naive() {
        history.nothing_ok_within(patience(period), now)
    } else {
        history.nothing_ok_within(period, start_of(day + Duration::days(1)))
    };

    due_and_missed.then_some(DayMark::Late)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::features::jobs::domain::JobCode;

    /// Friday 9 October 2026, noon UTC.
    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 9, 12, 0, 0).unwrap()
    }

    fn at(day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, day, hour, minute, 0)
            .unwrap()
    }

    fn job(code: &str, every_hours: Option<i32>) -> Job {
        Job::rehydrate(
            JobCode::new(code.to_string()).expect("code"),
            code.to_string(),
            None,
            every_hours,
        )
    }

    fn daily() -> Job {
        job("dams", Some(24))
    }

    fn on_demand() -> Job {
        job("farm_analysis", None)
    }

    fn message(text: &str) -> Option<RunMessage> {
        RunMessage::new(text.to_string()).expect("message")
    }

    fn run_of(
        job: &Job,
        started_at: DateTime<Utc>,
        took_minutes: Option<i64>,
        ok: Option<bool>,
        text: Option<&str>,
    ) -> JobRun {
        JobRun::rehydrate(
            1,
            job.code().clone(),
            started_at,
            took_minutes.map(|minutes| started_at + Duration::minutes(minutes)),
            ok,
            None,
            text.and_then(message),
        )
    }

    /// A run of the daily job that took ten minutes and went well.
    fn ok_at(started_at: DateTime<Utc>) -> JobRun {
        run_of(&daily(), started_at, Some(10), Some(true), None)
    }

    fn failed_at(started_at: DateTime<Utc>) -> JobRun {
        run_of(&daily(), started_at, Some(10), Some(false), Some("timeout"))
    }

    fn running_since(started_at: DateTime<Utc>) -> JobRun {
        run_of(&daily(), started_at, None, None, None)
    }

    fn judge(job: &Job, runs: &[JobRun]) -> JobStatus {
        let history = RunHistory::of(runs, history_since(std::slice::from_ref(job), now()));

        JobStatus::judge(job, &history, now())
    }

    fn days(status: &JobStatus) -> Vec<Option<DayMark>> {
        status.last_14_days().to_vec()
    }

    /// The mark of a day of October 2026 within the days shown.
    fn mark(status: &JobStatus, day: u32) -> Option<DayMark> {
        let index = DAYS_SHOWN - 1 - (9 - day) as usize;

        status.last_14_days()[index]
    }

    // The state.

    #[test]
    fn a_job_with_no_run_at_all_has_never_run() {
        let status = judge(&daily(), &[]);

        assert_eq!(*status.state(), JobState::Never);
        assert_eq!(*status.last_run(), None);
        assert_eq!(*status.last_ok(), None);
        assert_eq!(*status.next_due(), None);
        assert_eq!(*status.message(), None);
    }

    #[test]
    fn a_job_on_demand_that_never_ran_has_never_run_too() {
        assert_eq!(*judge(&on_demand(), &[]).state(), JobState::Never);
    }

    #[test]
    fn a_run_that_finished_ok_within_the_patience_is_ok() {
        let status = judge(&daily(), &[ok_at(at(9, 6, 0))]);

        assert_eq!(*status.state(), JobState::Ok);
    }

    #[test]
    fn exactly_one_and_a_half_periods_is_still_ok_and_a_minute_more_is_late() {
        // A daily job has 36 hours of patience. The run finished ten
        // minutes after it started.
        let on_the_edge = ok_at(now() - Duration::hours(36) - Duration::minutes(10));
        let just_over = ok_at(now() - Duration::hours(36) - Duration::minutes(11));

        assert_eq!(*judge(&daily(), &[on_the_edge]).state(), JobState::Ok);
        assert_eq!(*judge(&daily(), &[just_over]).state(), JobState::Late);
    }

    #[test]
    fn lateness_counts_from_when_the_run_finished_not_when_it_started() {
        // Started 40 hours ago, finished 30 hours ago: inside 36 hours.
        let slow = run_of(
            &daily(),
            now() - Duration::hours(40),
            Some(600),
            Some(true),
            None,
        );

        assert_eq!(*judge(&daily(), &[slow]).state(), JobState::Ok);
    }

    #[test]
    fn a_job_that_reported_only_an_old_run_is_late() {
        let status = judge(&daily(), &[ok_at(at(6, 6, 0))]);

        assert_eq!(*status.state(), JobState::Late);
        assert_eq!(*status.last_ok(), Some(at(6, 6, 10)));
    }

    #[test]
    fn the_period_sets_the_patience() {
        // Five hours ago is late for a job every 3 hours (4.5 hours of
        // patience) and fine for one every 12.
        let fires = job("fires", Some(3));
        let dryness = job("dryness", Some(12));
        let started_at = now() - Duration::hours(5);

        let fires_run = run_of(&fires, started_at, Some(0), Some(true), None);
        let dryness_run = run_of(&dryness, started_at, Some(0), Some(true), None);

        assert_eq!(*judge(&fires, &[fires_run]).state(), JobState::Late);
        assert_eq!(*judge(&dryness, &[dryness_run]).state(), JobState::Ok);
    }

    #[test]
    fn a_job_whose_last_finished_run_failed_has_failed_even_after_good_runs() {
        let status = judge(&daily(), &[ok_at(at(8, 6, 0)), failed_at(at(9, 6, 0))]);

        assert_eq!(*status.state(), JobState::Failed);
        assert_eq!(
            *status.last_ok(),
            Some(at(8, 6, 10)),
            "the last good run is still told"
        );
    }

    #[test]
    fn a_good_run_after_a_failure_clears_it() {
        let status = judge(&daily(), &[failed_at(at(9, 6, 0)), ok_at(at(9, 7, 0))]);

        assert_eq!(*status.state(), JobState::Ok);
    }

    #[test]
    fn a_failure_is_failed_rather_than_late_however_old_it_is() {
        let status = judge(&daily(), &[failed_at(at(2, 6, 0))]);

        assert_eq!(*status.state(), JobState::Failed);
    }

    #[test]
    fn a_run_still_going_does_not_hide_the_failure_before_it() {
        let status = judge(
            &daily(),
            &[failed_at(at(9, 6, 0)), running_since(at(9, 11, 55))],
        );

        assert_eq!(*status.state(), JobState::Failed);
        assert_eq!(
            status.last_run().as_ref().map(|run| *run.started_at()),
            Some(at(9, 11, 55)),
            "the last run is the one going now"
        );
        assert_eq!(
            status.message().as_ref().map(RunMessage::as_str),
            Some("timeout"),
            "the reason of the failure stays on screen"
        );
    }

    #[test]
    fn the_run_that_finished_last_decides_not_the_one_that_started_last() {
        // A long run started first and failed after a short one went well.
        let long_failure = run_of(&daily(), at(9, 6, 0), Some(120), Some(false), None);
        let short_success = run_of(&daily(), at(9, 7, 0), Some(5), Some(true), None);

        let status = judge(&daily(), &[long_failure, short_success]);

        assert_eq!(*status.state(), JobState::Failed);
    }

    #[test]
    fn a_scheduled_job_with_only_a_run_still_going_is_late() {
        // Nothing has finished ok, which is what late means.
        let status = judge(&daily(), &[running_since(at(9, 11, 55))]);

        assert_eq!(*status.state(), JobState::Late);
        assert_eq!(*status.last_ok(), None);
    }

    #[test]
    fn a_job_on_demand_is_never_late() {
        let old = run_of(&on_demand(), at(1, 6, 0), Some(10), Some(true), None);
        let going = run_of(&on_demand(), at(9, 11, 55), None, None, None);

        assert_eq!(*judge(&on_demand(), &[old]).state(), JobState::Ok);
        assert_eq!(*judge(&on_demand(), &[going]).state(), JobState::Ok);
    }

    #[test]
    fn a_job_on_demand_can_still_fail() {
        let failed = run_of(&on_demand(), at(9, 6, 0), Some(10), Some(false), None);

        assert_eq!(*judge(&on_demand(), &[failed]).state(), JobState::Failed);
    }

    // What is told next to the state.

    #[test]
    fn the_next_run_is_due_one_period_after_the_last_one_started() {
        let status = judge(&daily(), &[ok_at(at(8, 6, 0)), ok_at(at(9, 6, 0))]);

        assert_eq!(*status.next_due(), Some(at(10, 6, 0)));
    }

    #[test]
    fn a_job_on_demand_is_never_due() {
        let run = run_of(&on_demand(), at(9, 6, 0), Some(10), Some(true), None);

        assert_eq!(*judge(&on_demand(), &[run]).next_due(), None);
    }

    #[test]
    fn before_anything_has_finished_the_message_is_the_running_runs() {
        let going = run_of(&daily(), at(9, 11, 55), None, None, Some("fetching"));

        let status = judge(&daily(), &[going]);

        assert_eq!(
            status.message().as_ref().map(RunMessage::as_str),
            Some("fetching")
        );
    }

    #[test]
    fn the_message_is_the_last_finished_runs_also_when_it_went_well() {
        let earlier = run_of(
            &daily(),
            at(8, 6, 0),
            Some(10),
            Some(false),
            Some("timeout"),
        );
        let later = run_of(&daily(), at(9, 6, 0), Some(10), Some(true), Some("2 dams"));

        let status = judge(&daily(), &[earlier, later]);

        assert_eq!(
            status.message().as_ref().map(RunMessage::as_str),
            Some("2 dams")
        );
    }

    // The fourteen days.

    #[test]
    fn there_are_fourteen_days_oldest_first_ending_today() {
        // One good run on 1 October, one today.
        let status = judge(&daily(), &[ok_at(at(1, 6, 0)), ok_at(at(9, 6, 0))]);

        let days = days(&status);

        assert_eq!(days.len(), 14);
        assert_eq!(days[13], Some(DayMark::Ok), "today is the last entry");
        assert_eq!(
            mark(&status, 1),
            Some(DayMark::Ok),
            "1 October is eight days before today"
        );
        assert_eq!(days[5], Some(DayMark::Ok));
        assert_eq!(days[0], None, "26 September is before the first run");
    }

    #[test]
    fn a_job_that_never_ran_has_no_mark_on_any_day() {
        assert_eq!(days(&judge(&daily(), &[])), vec![None; 14]);
    }

    #[test]
    fn days_before_the_first_ever_run_have_no_mark() {
        let status = judge(&daily(), &[ok_at(at(7, 6, 0))]);

        assert!(
            days(&status)[..11].iter().all(Option::is_none),
            "a job is not late before it existed: {:?}",
            days(&status)
        );
        assert_eq!(mark(&status, 7), Some(DayMark::Ok));
    }

    #[test]
    fn a_day_is_failed_if_any_run_that_day_failed_even_beside_a_good_one() {
        let status = judge(
            &daily(),
            &[
                ok_at(at(8, 6, 0)),
                failed_at(at(8, 18, 0)),
                ok_at(at(9, 6, 0)),
            ],
        );

        assert_eq!(mark(&status, 8), Some(DayMark::Failed));
        assert_eq!(mark(&status, 9), Some(DayMark::Ok));
    }

    #[test]
    fn a_day_is_ok_with_at_least_one_good_run_and_no_failure() {
        // A job every 3 hours that ran once is still an ok day.
        let fires = job("fires", Some(3));
        let once = run_of(&fires, at(8, 6, 0), Some(5), Some(true), None);

        assert_eq!(mark(&judge(&fires, &[once]), 8), Some(DayMark::Ok));
    }

    #[test]
    fn a_past_day_with_no_run_is_late_for_a_daily_job() {
        let status = judge(&daily(), &[ok_at(at(6, 6, 0)), ok_at(at(9, 6, 0))]);

        assert_eq!(mark(&status, 6), Some(DayMark::Ok));
        assert_eq!(mark(&status, 7), Some(DayMark::Late));
        assert_eq!(mark(&status, 8), Some(DayMark::Late));
        assert_eq!(mark(&status, 9), Some(DayMark::Ok));
    }

    #[test]
    fn a_past_day_whose_only_run_never_finished_is_late() {
        let status = judge(&daily(), &[ok_at(at(6, 6, 0)), running_since(at(7, 6, 0))]);

        assert_eq!(mark(&status, 7), Some(DayMark::Late));
    }

    #[test]
    fn a_quiet_day_of_a_job_on_demand_has_no_mark() {
        let first = run_of(&on_demand(), at(5, 6, 0), Some(10), Some(true), None);
        let second = run_of(&on_demand(), at(8, 6, 0), Some(10), Some(false), None);

        let status = judge(&on_demand(), &[first, second]);

        assert_eq!(mark(&status, 5), Some(DayMark::Ok));
        assert_eq!(mark(&status, 6), None);
        assert_eq!(mark(&status, 7), None);
        assert_eq!(mark(&status, 8), Some(DayMark::Failed));
        assert_eq!(mark(&status, 9), None);
    }

    #[test]
    fn a_job_with_a_period_longer_than_a_day_is_not_late_on_the_days_between() {
        let every_three_days = job("survey", Some(72));
        let run = run_of(&every_three_days, at(3, 6, 0), Some(10), Some(true), None);

        let status = judge(&every_three_days, &[run]);

        assert_eq!(mark(&status, 3), Some(DayMark::Ok));
        assert_eq!(mark(&status, 4), None, "not due yet");
        assert_eq!(mark(&status, 5), None, "not due yet");
        assert_eq!(
            mark(&status, 6),
            Some(DayMark::Late),
            "by the end of 6 October, 72 hours had passed since 3 October 06:10"
        );
        assert_eq!(mark(&status, 8), Some(DayMark::Late));
    }

    #[test]
    fn today_is_not_late_while_the_job_still_has_time() {
        // Yesterday's run finished 30 hours ago: inside 36 hours.
        let status = judge(&daily(), &[ok_at(at(8, 6, 0))]);

        assert_eq!(mark(&status, 8), Some(DayMark::Ok));
        assert_eq!(mark(&status, 9), None, "today is not over");
        assert_eq!(*status.state(), JobState::Ok, "and the state agrees");
    }

    #[test]
    fn today_is_late_once_the_state_is_late() {
        // The last good run finished 54 hours ago.
        let status = judge(&daily(), &[ok_at(at(7, 6, 0))]);

        assert_eq!(mark(&status, 9), Some(DayMark::Late));
        assert_eq!(*status.state(), JobState::Late);
    }

    #[test]
    fn today_with_a_run_still_going_follows_the_same_patience() {
        let fresh = judge(
            &daily(),
            &[ok_at(at(8, 6, 0)), running_since(at(9, 11, 55))],
        );
        let overdue = judge(
            &daily(),
            &[ok_at(at(7, 6, 0)), running_since(at(9, 11, 55))],
        );

        assert_eq!(mark(&fresh, 9), None);
        assert_eq!(mark(&overdue, 9), Some(DayMark::Late));
    }

    #[test]
    fn a_run_belongs_to_the_day_it_started_on() {
        // Started at 23:50 on the 7th, finished ok at 00:10 on the 8th.
        let over_midnight = run_of(&daily(), at(7, 23, 50), Some(20), Some(true), None);

        let status = judge(&daily(), &[over_midnight]);

        assert_eq!(mark(&status, 7), Some(DayMark::Ok));
        assert_eq!(
            mark(&status, 8),
            None,
            "a good run finished within the 24 hours before the 8th ended"
        );
    }

    #[test]
    fn a_first_run_before_the_days_shown_makes_every_empty_day_late() {
        let long_ago = Utc.with_ymd_and_hms(2026, 9, 1, 6, 0, 0).unwrap();

        let status = judge(&daily(), &[ok_at(long_ago)]);

        assert_eq!(days(&status), vec![Some(DayMark::Late); 14]);
    }

    #[test]
    fn an_old_good_run_outside_the_loaded_runs_still_counts_through_the_last_ok() {
        // The history holds the last good run even when it is too old to be
        // among the recent ones, and it must not be read as "never ok".
        let weekly = job("survey", Some(24 * 30));
        let old = run_of(
            &weekly,
            Utc.with_ymd_and_hms(2026, 9, 20, 6, 0, 0).unwrap(),
            Some(10),
            Some(true),
            None,
        );
        let history = RunHistory::new(
            Some(*old.started_at()),
            Some(old.clone()),
            Some(old.clone()),
            Some(old),
            vec![],
        );

        let status = JobStatus::judge(&weekly, &history, now());

        assert_eq!(*status.state(), JobState::Ok);
        assert_eq!(days(&status), vec![None; 14]);
    }

    // What must be loaded.

    #[test]
    fn the_runs_loaded_reach_back_one_longest_period_before_the_first_day_shown() {
        let jobs = [job("fires", Some(3)), daily(), on_demand()];

        // The first day shown is 26 September; a day before is the 25th.
        assert_eq!(
            history_since(&jobs, now()),
            Utc.with_ymd_and_hms(2026, 9, 25, 0, 0, 0).unwrap()
        );
        assert_eq!(
            history_since(&[on_demand()], now()),
            Utc.with_ymd_and_hms(2026, 9, 26, 0, 0, 0).unwrap()
        );
    }

    #[test]
    fn a_history_is_picked_out_of_all_runs() {
        let runs = [
            ok_at(at(1, 6, 0)),
            ok_at(at(8, 6, 0)),
            failed_at(at(9, 6, 0)),
            running_since(at(9, 11, 55)),
        ];

        let history = RunHistory::of(&runs, at(8, 0, 0));

        assert_eq!(*history.first_started_at(), Some(at(1, 6, 0)));
        assert_eq!(
            history.latest().as_ref().map(|run| *run.started_at()),
            Some(at(9, 11, 55))
        );
        assert_eq!(
            history
                .latest_finished()
                .as_ref()
                .map(|run| *run.started_at()),
            Some(at(9, 6, 0))
        );
        assert_eq!(
            history.latest_ok().as_ref().map(|run| *run.started_at()),
            Some(at(8, 6, 0))
        );
        assert_eq!(history.recent().len(), 3);
    }
}
