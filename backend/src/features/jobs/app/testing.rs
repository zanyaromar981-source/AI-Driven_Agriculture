use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};

use crate::features::jobs::{
    app::{AppError, JobRepository},
    domain::{Job, JobCode, JobRun, RunHistory},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepositoryCall {
    FindJobs,
    FindJob {
        code: String,
    },
    UpsertRun {
        job: String,
        started_at: DateTime<Utc>,
        prune_before: DateTime<Utc>,
    },
    FindHistories {
        since: DateTime<Utc>,
    },
}

#[derive(Debug, Default)]
struct Script {
    jobs: Vec<Job>,
    runs: Vec<JobRun>,
    fail_with_database_error: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FakeJobRepository {
    script: Arc<Mutex<Script>>,
    calls: Arc<Mutex<Vec<RepositoryCall>>>,
}

impl FakeJobRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn holding(jobs: Vec<Job>, runs: Vec<JobRun>) -> Self {
        let fake = Self::new();
        {
            let mut script = fake.script.lock().expect("script lock");
            script.jobs = jobs;
            script.runs = runs;
        }
        fake
    }

    pub fn failing() -> Self {
        let fake = Self::new();
        fake.script
            .lock()
            .expect("script lock")
            .fail_with_database_error = true;
        fake
    }

    pub fn calls(&self) -> Vec<RepositoryCall> {
        self.calls.lock().expect("calls lock").clone()
    }

    pub fn runs(&self) -> Vec<JobRun> {
        self.script.lock().expect("script lock").runs.clone()
    }

    fn record(&self, call: RepositoryCall) {
        self.calls.lock().expect("calls lock").push(call);
    }

    fn guard(&self) -> Result<(), AppError> {
        if self
            .script
            .lock()
            .expect("script lock")
            .fail_with_database_error
        {
            return Err(crate::app::AppError::DatabaseError("fake".to_string()).into());
        }

        Ok(())
    }
}

#[async_trait]
impl JobRepository for FakeJobRepository {
    async fn find_jobs(&self) -> Result<Vec<Job>, AppError> {
        self.record(RepositoryCall::FindJobs);
        self.guard()?;

        Ok(self.script.lock().expect("script lock").jobs.clone())
    }

    async fn find_job(&self, code: &JobCode) -> Result<Option<Job>, AppError> {
        self.record(RepositoryCall::FindJob {
            code: String::from(code),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script.jobs.iter().find(|job| job.code() == code).cloned())
    }

    async fn upsert_run(
        &self,
        run: &JobRun,
        prune_before: DateTime<Utc>,
    ) -> Result<JobRun, AppError> {
        self.record(RepositoryCall::UpsertRun {
            job: String::from(run.job()),
            started_at: *run.started_at(),
            prune_before,
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        let position = script.runs.iter().position(|stored| {
            stored.job() == run.job() && stored.started_at() == run.started_at()
        });

        let stored = match position {
            Some(position) => {
                let existing = script.runs[position].clone();

                // As in the real repository: "still going" never replaces
                // "finished".
                if existing.finished_at().is_some() && run.finished_at().is_none() {
                    existing
                } else {
                    let replaced = JobRun::rehydrate(
                        existing.id().unwrap_or_default(),
                        run.job().clone(),
                        *run.started_at(),
                        *run.finished_at(),
                        *run.ok(),
                        *run.rows(),
                        run.message().clone(),
                    );
                    script.runs[position] = replaced.clone();
                    replaced
                }
            }
            None => {
                let id = script
                    .runs
                    .iter()
                    .filter_map(|stored| *stored.id())
                    .max()
                    .unwrap_or(0)
                    + 1;
                let created = JobRun::rehydrate(
                    id,
                    run.job().clone(),
                    *run.started_at(),
                    *run.finished_at(),
                    *run.ok(),
                    *run.rows(),
                    run.message().clone(),
                );
                script.runs.push(created.clone());
                created
            }
        };

        script
            .runs
            .retain(|kept| kept.job() != run.job() || *kept.started_at() >= prune_before);

        Ok(stored)
    }

    async fn find_histories(
        &self,
        since: DateTime<Utc>,
    ) -> Result<HashMap<JobCode, RunHistory>, AppError> {
        self.record(RepositoryCall::FindHistories { since });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let mut by_job: HashMap<JobCode, Vec<JobRun>> = HashMap::new();
        for run in &script.runs {
            by_job
                .entry(run.job().clone())
                .or_default()
                .push(run.clone());
        }

        Ok(by_job
            .into_iter()
            .map(|(code, runs)| (code, RunHistory::of(&runs, since)))
            .collect())
    }
}

/// Friday 9 October 2026, noon UTC.
pub fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 9, 12, 0, 0).unwrap()
}

/// A moment of October 2026.
pub fn at(day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, day, hour, minute, 0)
        .unwrap()
}

pub fn code(value: &str) -> JobCode {
    JobCode::new(value.to_string()).expect("code")
}

fn a_job(value: &str, every_hours: Option<i32>) -> Job {
    Job::rehydrate(code(value), value.to_string(), None, every_hours)
}

/// Every 24 hours.
pub fn dams() -> Job {
    a_job("dams", Some(24))
}

/// Every 3 hours.
pub fn fires() -> Job {
    a_job("fires", Some(3))
}

/// On demand.
pub fn farm_analysis() -> Job {
    a_job("farm_analysis", None)
}

/// A stored run that took ten minutes. `ok` None = still going.
pub fn a_run(id: i32, job: &Job, started_at: DateTime<Utc>, ok: Option<bool>) -> JobRun {
    JobRun::rehydrate(
        id,
        job.code().clone(),
        started_at,
        ok.map(|_| started_at + chrono::Duration::minutes(10)),
        ok,
        None,
        None,
    )
}
