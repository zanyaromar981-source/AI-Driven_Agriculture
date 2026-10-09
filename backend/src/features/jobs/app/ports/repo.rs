use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::features::jobs::{
    app::AppError,
    domain::{Job, JobCode, JobRun, RunHistory},
};

/// There is no way to add, change or remove a job here: jobs exist by
/// migration, because each one is a program that reports under its own name.
#[async_trait]
pub trait JobRepository: Send + Sync + std::fmt::Debug {
    /// Every job, ordered by code.
    async fn find_jobs(&self) -> Result<Vec<Job>, AppError>;

    async fn find_job(&self, code: &JobCode) -> Result<Option<Job>, AppError>;

    /// Stores the run for its job and start, replacing the report already
    /// there if the job has reported that run before, with one exception: a
    /// report that says the run is still going never replaces one that says
    /// it finished, so a late or repeated "started" cannot undo the result.
    /// In the same transaction the job's runs that started before
    /// `prune_before` are removed. Returns the run as it is stored after.
    async fn upsert_run(
        &self,
        run: &JobRun,
        prune_before: DateTime<Utc>,
    ) -> Result<JobRun, AppError>;

    /// What is known of each job's runs: the first, the latest, the latest
    /// finished and the latest finished ok out of all of them, and every run
    /// that started at or after `since`. A job with no run has no entry.
    async fn find_histories(
        &self,
        since: DateTime<Utc>,
    ) -> Result<HashMap<JobCode, RunHistory>, AppError>;
}
