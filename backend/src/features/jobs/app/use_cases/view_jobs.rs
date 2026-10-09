use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::features::jobs::{
    app::{AppError, JobRepository},
    domain::{JobStatus, RunHistory, history_since},
};

pub struct ViewJobsUseCase {
    repository: Arc<dyn JobRepository>,
}

impl ViewJobsUseCase {
    pub fn new(repository: Arc<dyn JobRepository>) -> Self {
        Self { repository }
    }

    /// Returns every job with how it stands at `now` and how its last
    /// fourteen days went. The judging itself is the domain's.
    pub async fn execute(&self, now: DateTime<Utc>) -> Result<Vec<JobStatus>, AppError> {
        let jobs = self.repository.find_jobs().await?;

        let mut histories = self
            .repository
            .find_histories(history_since(&jobs, now))
            .await?;

        let statuses: Vec<JobStatus> = jobs
            .iter()
            .map(|job| {
                let history = histories
                    .remove(job.code())
                    .unwrap_or_else(RunHistory::empty);

                JobStatus::judge(job, &history, now)
            })
            .collect();

        tracing::debug!(returned = statuses.len(), "job statuses listed");

        Ok(statuses)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone};

    use super::*;
    use crate::features::jobs::{
        app::testing::{
            FakeJobRepository, RepositoryCall, a_run, at, dams, farm_analysis, fires, now,
        },
        domain::{DayMark, JobState},
    };

    fn state_of(statuses: &[JobStatus], job: &str) -> JobState {
        *statuses
            .iter()
            .find(|status| status.job().code().as_str() == job)
            .unwrap_or_else(|| panic!("no status for {job}"))
            .state()
    }

    #[tokio::test]
    async fn every_job_is_judged_from_its_own_runs() {
        let repository = FakeJobRepository::holding(
            vec![dams(), farm_analysis(), fires()],
            vec![
                // Dams ran well this morning.
                a_run(1, &dams(), at(9, 6, 0), Some(true)),
                // Fires ran well, but six hours ago: late for every 3 hours.
                a_run(2, &fires(), at(9, 6, 0), Some(true)),
            ],
        );
        let use_case = ViewJobsUseCase::new(Arc::new(repository.clone()));

        let statuses = use_case.execute(now()).await.expect("statuses");

        assert_eq!(statuses.len(), 3, "a job with no run is still listed");
        assert_eq!(state_of(&statuses, "dams"), JobState::Ok);
        assert_eq!(state_of(&statuses, "fires"), JobState::Late);
        assert_eq!(state_of(&statuses, "farm_analysis"), JobState::Never);
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindJobs,
                RepositoryCall::FindHistories {
                    // One day, the longest period, before the first of the
                    // fourteen days: 26 September.
                    since: Utc.with_ymd_and_hms(2026, 9, 25, 0, 0, 0).unwrap(),
                },
            ],
            "the runs of all jobs are read at once, not job by job"
        );
    }

    #[tokio::test]
    async fn the_jobs_come_back_in_the_order_they_are_stored() {
        let repository = FakeJobRepository::holding(vec![dams(), farm_analysis(), fires()], vec![]);
        let use_case = ViewJobsUseCase::new(Arc::new(repository));

        let statuses = use_case.execute(now()).await.expect("statuses");

        assert_eq!(
            statuses
                .iter()
                .map(|status| status.job().code().as_str())
                .collect::<Vec<_>>(),
            vec!["dams", "farm_analysis", "fires"]
        );
    }

    #[tokio::test]
    async fn a_failure_shows_in_the_state_and_on_its_day() {
        let repository = FakeJobRepository::holding(
            vec![dams()],
            vec![
                a_run(1, &dams(), at(8, 6, 0), Some(true)),
                a_run(2, &dams(), at(9, 6, 0), Some(false)),
            ],
        );
        let use_case = ViewJobsUseCase::new(Arc::new(repository));

        let statuses = use_case.execute(now()).await.expect("statuses");
        let days = statuses[0].last_14_days();

        assert_eq!(*statuses[0].state(), JobState::Failed);
        assert_eq!(days[13], Some(DayMark::Failed));
        assert_eq!(days[12], Some(DayMark::Ok));
        assert_eq!(days[11], None, "before the first run");
    }

    #[tokio::test]
    async fn the_same_runs_are_judged_differently_as_time_passes() {
        let repository = FakeJobRepository::holding(
            vec![dams()],
            vec![a_run(1, &dams(), at(9, 6, 0), Some(true))],
        );
        let use_case = ViewJobsUseCase::new(Arc::new(repository));

        let soon = use_case.execute(now()).await.expect("statuses");
        let much_later = use_case
            .execute(now() + Duration::days(3))
            .await
            .expect("statuses");

        assert_eq!(*soon[0].state(), JobState::Ok);
        assert_eq!(*much_later[0].state(), JobState::Late);
    }

    #[tokio::test]
    async fn with_no_jobs_there_is_nothing_to_show() {
        let use_case = ViewJobsUseCase::new(Arc::new(FakeJobRepository::new()));

        assert!(use_case.execute(now()).await.expect("statuses").is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ViewJobsUseCase::new(Arc::new(FakeJobRepository::failing()));

        assert!(use_case.execute(now()).await.is_err());
    }
}
