use sea_orm::ActiveValue::{NotSet, Set};

use crate::features::jobs::{
    app::AppError,
    domain::{Job, JobCode, JobRun, RunMessage},
    infra::persistence::postgres::entities::{job_runs, jobs},
};

impl TryFrom<jobs::Model> for Job {
    type Error = AppError;

    fn try_from(model: jobs::Model) -> Result<Self, Self::Error> {
        Ok(Job::rehydrate(
            JobCode::new(model.job)?,
            model.name_en,
            model.name_ku,
            model.every_hours,
        ))
    }
}

impl TryFrom<job_runs::Model> for JobRun {
    type Error = AppError;

    fn try_from(model: job_runs::Model) -> Result<Self, Self::Error> {
        Ok(JobRun::rehydrate(
            model.id,
            JobCode::new(model.job)?,
            model.started_at.and_utc(),
            model.finished_at.map(|finished_at| finished_at.and_utc()),
            model.ok,
            model.rows,
            model.message.map(RunMessage::new).transpose()?.flatten(),
        ))
    }
}

impl From<&JobRun> for job_runs::ActiveModel {
    fn from(run: &JobRun) -> Self {
        job_runs::ActiveModel {
            id: match *run.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            job: Set(run.job().into()),
            started_at: Set(run.started_at().naive_utc()),
            finished_at: Set(run.finished_at().map(|finished_at| finished_at.naive_utc())),
            ok: Set(*run.ok()),
            rows: Set(*run.rows()),
            message: Set(run.message().as_ref().map(String::from)),
        }
    }
}
