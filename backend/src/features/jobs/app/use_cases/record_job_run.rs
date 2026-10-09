use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::features::jobs::{
    app::{AppError, JobRepository},
    domain::{JobCode, JobRun, RunMessage, prune_before},
};

pub struct RecordJobRunInput {
    pub job: JobCode,
    pub started_at: DateTime<Utc>,
    /// None while the run is still going.
    pub finished_at: Option<DateTime<Utc>>,
    pub ok: Option<bool>,
    pub rows: Option<i64>,
    pub message: Option<RunMessage>,
}

pub struct RecordJobRunUseCase {
    repository: Arc<dyn JobRepository>,
}

impl RecordJobRunUseCase {
    pub fn new(repository: Arc<dyn JobRepository>) -> Self {
        Self { repository }
    }

    /// Stores what a job reports about one of its runs. A job usually
    /// reports a run twice, when it starts and when it ends, and may repeat
    /// either: the job and the start are the key, so each report replaces
    /// the one before. The same write removes the job's runs that are too
    /// old to keep.
    pub async fn execute(
        &self,
        input: RecordJobRunInput,
        now: DateTime<Utc>,
    ) -> Result<JobRun, AppError> {
        let job = self
            .repository
            .find_job(&input.job)
            .await?
            .ok_or_else(|| AppError::JobNotFound(String::from(&input.job)))?;

        let run = JobRun::new(
            &job,
            input.started_at,
            input.finished_at,
            input.ok,
            input.rows,
            input.message,
            now,
        )?;

        let stored = self.repository.upsert_run(&run, prune_before(now)).await?;

        tracing::info!(
            job = input.job.as_str(),
            started_at = %stored.started_at(),
            finished = stored.finished_at().is_some(),
            ok = *stored.ok(),
            rows = *stored.rows(),
            "job run recorded"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::features::jobs::{
        app::testing::{FakeJobRepository, RepositoryCall, a_run, at, code, dams, fires, now},
        domain::JobError,
    };

    fn started(job: &str, started_at: DateTime<Utc>) -> RecordJobRunInput {
        RecordJobRunInput {
            job: code(job),
            started_at,
            finished_at: None,
            ok: None,
            rows: None,
            message: None,
        }
    }

    fn finished(job: &str, started_at: DateTime<Utc>, ok: bool, rows: i64) -> RecordJobRunInput {
        RecordJobRunInput {
            job: code(job),
            started_at,
            finished_at: Some(started_at + Duration::minutes(4)),
            ok: Some(ok),
            rows: Some(rows),
            message: RunMessage::new(format!("{rows} rows")).expect("message"),
        }
    }

    fn repository() -> FakeJobRepository {
        FakeJobRepository::holding(vec![dams(), fires()], vec![])
    }

    #[tokio::test]
    async fn a_first_report_stores_the_run_as_still_going() {
        let repository = repository();
        let use_case = RecordJobRunUseCase::new(Arc::new(repository.clone()));

        let run = use_case
            .execute(started("fires", at(9, 11, 50)), now())
            .await
            .expect("run");

        assert!(run.id().is_some());
        assert_eq!(*run.finished_at(), None);
        assert_eq!(*run.ok(), None);
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindJob {
                    code: "fires".to_string()
                },
                RepositoryCall::UpsertRun {
                    job: "fires".to_string(),
                    started_at: at(9, 11, 50),
                    prune_before: now() - Duration::days(60),
                },
            ]
        );
    }

    #[tokio::test]
    async fn the_closing_report_replaces_the_first_instead_of_adding_a_run() {
        let repository = repository();
        let use_case = RecordJobRunUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(started("fires", at(9, 11, 50)), now())
            .await
            .expect("run");
        let run = use_case
            .execute(finished("fires", at(9, 11, 50), true, 312), now())
            .await
            .expect("run");

        assert_eq!(*run.ok(), Some(true));
        assert_eq!(*run.rows(), Some(312));
        assert_eq!(repository.runs().len(), 1, "one run, reported twice");
    }

    #[tokio::test]
    async fn the_same_report_twice_stores_one_run_with_the_same_answer() {
        let repository = repository();
        let use_case = RecordJobRunUseCase::new(Arc::new(repository.clone()));

        let first = use_case
            .execute(finished("fires", at(9, 11, 50), true, 312), now())
            .await
            .expect("run");
        let second = use_case
            .execute(finished("fires", at(9, 11, 50), true, 312), now())
            .await
            .expect("run");

        assert_eq!(first, second);
        assert_eq!(repository.runs().len(), 1);
    }

    #[tokio::test]
    async fn a_repeated_start_does_not_undo_the_result() {
        let repository = repository();
        let use_case = RecordJobRunUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(finished("fires", at(9, 11, 50), true, 312), now())
            .await
            .expect("run");
        let run = use_case
            .execute(started("fires", at(9, 11, 50)), now())
            .await
            .expect("run");

        assert_eq!(*run.ok(), Some(true), "the answer is the stored result");
        assert_eq!(*repository.runs()[0].rows(), Some(312));
    }

    #[tokio::test]
    async fn two_starts_are_two_runs() {
        let repository = repository();
        let use_case = RecordJobRunUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(started("fires", at(9, 8, 50)), now())
            .await
            .expect("run");
        use_case
            .execute(started("fires", at(9, 11, 50)), now())
            .await
            .expect("run");

        assert_eq!(repository.runs().len(), 2);
    }

    #[tokio::test]
    async fn the_same_start_under_another_job_is_another_run() {
        let repository = repository();
        let use_case = RecordJobRunUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(started("fires", at(9, 11, 50)), now())
            .await
            .expect("run");
        use_case
            .execute(started("dams", at(9, 11, 50)), now())
            .await
            .expect("run");

        assert_eq!(repository.runs().len(), 2);
    }

    #[tokio::test]
    async fn a_report_removes_that_jobs_runs_older_than_sixty_days_and_no_others() {
        let long_ago = now() - Duration::days(61);
        let repository = FakeJobRepository::holding(
            vec![dams(), fires()],
            vec![
                a_run(1, &fires(), long_ago, Some(true)),
                a_run(2, &fires(), now() - Duration::days(59), Some(true)),
                a_run(3, &dams(), long_ago, Some(true)),
            ],
        );
        let use_case = RecordJobRunUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(started("fires", at(9, 11, 50)), now())
            .await
            .expect("run");

        let kept: Vec<i32> = repository
            .runs()
            .iter()
            .filter_map(|run| *run.id())
            .collect();

        assert!(!kept.contains(&1), "the old fires run is gone");
        assert!(kept.contains(&2), "a run inside sixty days stays");
        assert!(
            kept.contains(&3),
            "another job's old run is not this report's to remove"
        );
    }

    #[tokio::test]
    async fn an_unknown_job_is_not_found_and_nothing_is_written() {
        let repository = repository();
        let use_case = RecordJobRunUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(started("floods", at(9, 11, 50)), now())
            .await;

        assert!(matches!(result, Err(AppError::JobNotFound(job)) if job == "floods"));
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindJob {
                code: "floods".to_string()
            }]
        );
    }

    #[tokio::test]
    async fn a_run_that_cannot_be_is_refused_and_nothing_is_written() {
        let repository = repository();
        let use_case = RecordJobRunUseCase::new(Arc::new(repository.clone()));

        let mut backwards = finished("fires", at(9, 11, 50), true, 1);
        backwards.finished_at = Some(at(9, 11, 40));

        let result = use_case.execute(backwards, now()).await;

        assert!(matches!(
            result,
            Err(AppError::Job(JobError::FinishesBeforeItStarts))
        ));
        assert!(
            !repository
                .calls()
                .iter()
                .any(|call| matches!(call, RepositoryCall::UpsertRun { .. })),
            "the refused run must not be written"
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = RecordJobRunUseCase::new(Arc::new(FakeJobRepository::failing()));

        assert!(
            use_case
                .execute(started("fires", at(9, 11, 50)), now())
                .await
                .is_err()
        );
    }
}
