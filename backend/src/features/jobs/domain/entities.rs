use chrono::{DateTime, Duration, Utc};
use getset::Getters;

use crate::features::jobs::domain::{JobCode, JobError, RunMessage};

/// Runs older than this are removed when their job next reports, so the
/// table cannot grow without end. It is far more than the fourteen days the
/// dashboard shows.
pub const RETENTION_DAYS: i64 = 60;

/// A job's clock may run a little ahead of the server's.
const CLOCK_SLACK_MINUTES: i64 = 5;

/// Runs that started before this moment are no longer kept.
pub fn prune_before(now: DateTime<Utc>) -> DateTime<Utc> {
    now - Duration::days(RETENTION_DAYS)
}

/// One automatic data job. Jobs exist only by migration, because each one
/// is a program that reports under its own name.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct Job {
    code: JobCode,
    name_en: String,
    name_ku: Option<String>,
    /// How often the job is meant to run. None = on demand, so never late.
    every_hours: Option<i32>,
}

impl Job {
    /// Reconstruct from persisted state.
    pub fn rehydrate(
        code: JobCode,
        name_en: String,
        name_ku: Option<String>,
        every_hours: Option<i32>,
    ) -> Self {
        Self {
            code,
            name_en,
            name_ku,
            every_hours,
        }
    }

    /// How often the job is meant to run, when it has a schedule at all.
    pub fn period(&self) -> Option<Duration> {
        self.every_hours
            .filter(|hours| *hours > 0)
            .map(|hours| Duration::hours(i64::from(hours)))
    }
}

/// One run of a job, as the job itself reported it. The job and the moment
/// it started are its key.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct JobRun {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    job: JobCode,
    started_at: DateTime<Utc>,
    /// None while the run is still going.
    finished_at: Option<DateTime<Utc>>,
    /// None while the run is still going.
    ok: Option<bool>,
    /// How many rows the run wrote, when the job counts them.
    rows: Option<i64>,
    message: Option<RunMessage>,
}

impl JobRun {
    pub fn new(
        job: &Job,
        started_at: DateTime<Utc>,
        finished_at: Option<DateTime<Utc>>,
        ok: Option<bool>,
        rows: Option<i64>,
        message: Option<RunMessage>,
        now: DateTime<Utc>,
    ) -> Result<Self, JobError> {
        if started_at > now + Duration::minutes(CLOCK_SLACK_MINUTES) {
            return Err(JobError::StartsInTheFuture);
        }

        // A run this old would be removed by the very report that stores it.
        if started_at < prune_before(now) {
            return Err(JobError::TooOld {
                days: RETENTION_DAYS,
            });
        }

        if finished_at.is_some_and(|finished_at| finished_at < started_at) {
            return Err(JobError::FinishesBeforeItStarts);
        }

        // "Finished" is told by `finished_at` alone everywhere else, so a
        // run with an outcome but no end, or an end but no outcome, would be
        // judged wrongly.
        if finished_at.is_some() != ok.is_some() {
            return Err(JobError::HalfFinished);
        }

        if rows.is_some_and(|rows| rows < 0) {
            return Err(JobError::NegativeRows);
        }

        Ok(Self {
            id: None,
            job: job.code().clone(),
            started_at,
            finished_at,
            ok,
            rows,
            message,
        })
    }

    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        job: JobCode,
        started_at: DateTime<Utc>,
        finished_at: Option<DateTime<Utc>>,
        ok: Option<bool>,
        rows: Option<i64>,
        message: Option<RunMessage>,
    ) -> Self {
        Self {
            id: Some(id),
            job,
            started_at,
            finished_at,
            ok,
            rows,
            message,
        }
    }

    /// When the run finished well. None while it is going or if it failed.
    pub fn finished_ok_at(&self) -> Option<DateTime<Utc>> {
        self.finished_at.filter(|_| self.ok == Some(true))
    }

    /// The run is over and did not go well.
    pub fn failed(&self) -> bool {
        self.finished_at.is_some() && self.ok != Some(true)
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 9, 12, 0, 0).unwrap()
    }

    fn fires() -> Job {
        Job::rehydrate(
            JobCode::new("fires".to_string()).expect("code"),
            "Fires".to_string(),
            None,
            Some(3),
        )
    }

    fn run(
        started_ago: Duration,
        finished_ago: Option<Duration>,
        ok: Option<bool>,
        rows: Option<i64>,
    ) -> Result<JobRun, JobError> {
        JobRun::new(
            &fires(),
            now() - started_ago,
            finished_ago.map(|ago| now() - ago),
            ok,
            rows,
            None,
            now(),
        )
    }

    #[test]
    fn a_run_still_going_has_neither_an_end_nor_an_outcome() {
        let run = run(Duration::minutes(2), None, None, None).expect("run");

        assert_eq!(*run.id(), None);
        assert_eq!(run.job().as_str(), "fires");
        assert_eq!(run.finished_ok_at(), None);
        assert!(!run.failed(), "a run still going has not failed");
    }

    #[test]
    fn a_run_that_finished_well_says_when() {
        let run = run(
            Duration::minutes(10),
            Some(Duration::minutes(8)),
            Some(true),
            Some(312),
        )
        .expect("run");

        assert_eq!(run.finished_ok_at(), Some(now() - Duration::minutes(8)));
        assert!(!run.failed());
    }

    #[test]
    fn a_run_that_finished_badly_has_failed() {
        let run = run(
            Duration::minutes(10),
            Some(Duration::minutes(8)),
            Some(false),
            None,
        )
        .expect("run");

        assert!(run.failed());
        assert_eq!(run.finished_ok_at(), None);
    }

    #[test]
    fn an_end_before_the_start_is_refused() {
        assert!(matches!(
            run(
                Duration::minutes(10),
                Some(Duration::minutes(11)),
                Some(true),
                None
            ),
            Err(JobError::FinishesBeforeItStarts)
        ));
    }

    #[test]
    fn a_run_may_end_in_the_instant_it_started() {
        assert!(
            run(
                Duration::minutes(10),
                Some(Duration::minutes(10)),
                Some(true),
                None
            )
            .is_ok()
        );
    }

    #[test]
    fn an_end_without_an_outcome_or_an_outcome_without_an_end_is_refused() {
        assert!(matches!(
            run(
                Duration::minutes(10),
                Some(Duration::minutes(8)),
                None,
                None
            ),
            Err(JobError::HalfFinished)
        ));
        assert!(matches!(
            run(Duration::minutes(10), None, Some(true), None),
            Err(JobError::HalfFinished)
        ));
    }

    #[test]
    fn a_start_in_the_future_is_refused_beyond_a_little_clock_slack() {
        assert!(run(Duration::minutes(-4), None, None, None).is_ok());
        assert!(matches!(
            run(Duration::minutes(-6), None, None, None),
            Err(JobError::StartsInTheFuture)
        ));
    }

    #[test]
    fn a_run_older_than_what_is_kept_is_refused() {
        assert!(run(Duration::days(RETENTION_DAYS), None, None, None).is_ok());
        assert!(matches!(
            run(
                Duration::days(RETENTION_DAYS) + Duration::seconds(1),
                None,
                None,
                None
            ),
            Err(JobError::TooOld { days: 60 })
        ));
    }

    #[test]
    fn a_negative_row_count_is_refused_and_zero_is_fine() {
        assert!(matches!(
            run(
                Duration::minutes(10),
                Some(Duration::minutes(8)),
                Some(true),
                Some(-1)
            ),
            Err(JobError::NegativeRows)
        ));
        assert!(
            run(
                Duration::minutes(10),
                Some(Duration::minutes(8)),
                Some(true),
                Some(0)
            )
            .is_ok()
        );
    }

    #[test]
    fn runs_are_kept_for_sixty_days() {
        assert_eq!(prune_before(now()), now() - Duration::days(60));
    }

    #[test]
    fn a_job_on_demand_has_no_period() {
        let on_demand = Job::rehydrate(
            JobCode::new("farm_analysis".to_string()).expect("code"),
            "Farm analysis".to_string(),
            None,
            None,
        );

        assert_eq!(on_demand.period(), None);
        assert_eq!(fires().period(), Some(Duration::hours(3)));
    }
}
