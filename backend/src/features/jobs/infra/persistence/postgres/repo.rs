use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    TransactionTrait, TryInsertResult,
    sea_query::{Expr, ExprTrait, OnConflict},
};

use crate::{
    app::AppError as GlobalAppError,
    features::jobs::{
        app::{AppError, JobRepository},
        domain::{Job, JobCode, JobRun, RunHistory},
        infra::persistence::postgres::entities::{job_runs, jobs},
    },
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "job repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

/// One run per job out of a query that returns at most one row per job.
fn by_job(models: Vec<job_runs::Model>) -> Result<HashMap<JobCode, JobRun>, AppError> {
    models
        .into_iter()
        .map(|model| {
            let run = JobRun::try_from(model)?;

            Ok((run.job().clone(), run))
        })
        .collect()
}

#[derive(Debug)]
pub struct JobPostgresRepository {
    conn: DatabaseConnection,
}

impl JobPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl JobRepository for JobPostgresRepository {
    async fn find_jobs(&self) -> Result<Vec<Job>, AppError> {
        let models = jobs::Entity::find()
            .order_by_asc(jobs::Column::Job)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(Job::try_from).collect()
    }

    async fn find_job(&self, code: &JobCode) -> Result<Option<Job>, AppError> {
        let model = jobs::Entity::find_by_id(code.as_str().to_string())
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        model.map(Job::try_from).transpose()
    }

    async fn upsert_run(
        &self,
        run: &JobRun,
        prune_before: DateTime<Utc>,
    ) -> Result<JobRun, AppError> {
        let transaction = self.conn.begin().await.map_err(database_error)?;

        // One statement decides. The job and the start are the key, so a
        // repeated report replaces the earlier one instead of adding a row,
        // and two reports at the same moment end as one row. The condition
        // keeps a finished run finished: a "still going" report that arrives
        // late, or again, changes nothing.
        let upserted = job_runs::Entity::insert(job_runs::ActiveModel::from(run))
            .on_conflict(
                OnConflict::columns([job_runs::Column::Job, job_runs::Column::StartedAt])
                    .update_columns([
                        job_runs::Column::FinishedAt,
                        job_runs::Column::Ok,
                        job_runs::Column::Rows,
                        job_runs::Column::Message,
                    ])
                    .action_and_where(
                        Expr::col((job_runs::Entity, job_runs::Column::FinishedAt))
                            .is_null()
                            .or(Expr::cust("\"excluded\".\"finished_at\" IS NOT NULL")),
                    )
                    .to_owned(),
            )
            .try_insert()
            .exec_with_returning_many(&transaction)
            .await
            .map_err(database_error)?;

        let written = match upserted {
            TryInsertResult::Inserted(mut models) => models.pop(),
            TryInsertResult::Conflicted | TryInsertResult::Empty => None,
        };

        // No row back means the condition held the stored result in place:
        // the answer is that stored run.
        let model = match written {
            Some(model) => model,
            None => job_runs::Entity::find()
                .filter(job_runs::Column::Job.eq(run.job().as_str()))
                .filter(job_runs::Column::StartedAt.eq(run.started_at().naive_utc()))
                .one(&transaction)
                .await
                .map_err(database_error)?
                .ok_or(GlobalAppError::InternalServerError)?,
        };

        // Keeps the table from growing without end. Only this job's runs:
        // a job that stopped reporting keeps its last runs on show.
        job_runs::Entity::delete_many()
            .filter(job_runs::Column::Job.eq(run.job().as_str()))
            .filter(job_runs::Column::StartedAt.lt(prune_before.naive_utc()))
            .exec(&transaction)
            .await
            .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)?;

        JobRun::try_from(model)
    }

    async fn find_histories(
        &self,
        since: DateTime<Utc>,
    ) -> Result<HashMap<JobCode, RunHistory>, AppError> {
        // Four runs are picked out of each job's whole history, each by one
        // query that returns one row per job, so the cost does not grow with
        // the number of jobs.
        let one_per_job = || {
            job_runs::Entity::find()
                .distinct_on([job_runs::Column::Job])
                .order_by_asc(job_runs::Column::Job)
        };

        let first = one_per_job()
            .order_by_asc(job_runs::Column::StartedAt)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let latest = one_per_job()
            .order_by_desc(job_runs::Column::StartedAt)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let latest_finished = one_per_job()
            .filter(job_runs::Column::FinishedAt.is_not_null())
            .order_by_desc(job_runs::Column::FinishedAt)
            .order_by_desc(job_runs::Column::StartedAt)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let latest_ok = one_per_job()
            .filter(job_runs::Column::FinishedAt.is_not_null())
            .filter(job_runs::Column::Ok.eq(true))
            .order_by_desc(job_runs::Column::FinishedAt)
            .order_by_desc(job_runs::Column::StartedAt)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let recent = job_runs::Entity::find()
            .filter(job_runs::Column::StartedAt.gte(since.naive_utc()))
            .order_by_asc(job_runs::Column::Job)
            .order_by_asc(job_runs::Column::StartedAt)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let first = by_job(first)?;
        let mut latest = by_job(latest)?;
        let mut latest_finished = by_job(latest_finished)?;
        let mut latest_ok = by_job(latest_ok)?;

        let mut recent_by_job: HashMap<JobCode, Vec<JobRun>> = HashMap::new();
        for model in recent {
            let run = JobRun::try_from(model)?;

            recent_by_job
                .entry(run.job().clone())
                .or_default()
                .push(run);
        }

        // Every job that ever ran has a first run, so the first runs name
        // the jobs that have a history.
        Ok(first
            .into_iter()
            .map(|(code, first)| {
                let history = RunHistory::new(
                    Some(*first.started_at()),
                    latest.remove(&code),
                    latest_finished.remove(&code),
                    latest_ok.remove(&code),
                    recent_by_job.remove(&code).unwrap_or_default(),
                );

                (code, history)
            })
            .collect())
    }
}
