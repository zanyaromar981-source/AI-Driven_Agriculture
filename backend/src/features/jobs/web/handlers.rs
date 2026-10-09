use axum::extract::{Path, State};
use axum_extra::extract::WithRejection;
use chrono::Utc;

use super::{
    dtos::{JobStatusResponse, JobsResponse, RecordJobRunParams, SavedJobRunResponse},
    errors::WebError,
};

use crate::{
    features::jobs::domain::JobCode,
    infra::http::{ApiResponse, ErrorBody, ValidatedJson},
    shared::AppState,
};

/// A code that is not well formed cannot name a job, so it is not found
/// rather than a bad request.
fn job_code(raw: String) -> Result<JobCode, WebError> {
    JobCode::new(raw).map_err(|_| WebError::not_found())
}

/// Store what a job reports about one of its runs, replacing its earlier report of that run
#[utoipa::path(
    put,
    path = "/v1/ingest/jobs/{job}/runs/{started_at}",
    tag = "jobs",
    params(
        ("job" = String, Path, description = "Job code, for example fires"),
        ("started_at" = String, Path, description = "When the run started, an RFC 3339 timestamp, URL-encoded"),
        ("X-Service-Key" = String, Header, description = "The data jobs' shared key")
    ),
    request_body = RecordJobRunParams,
    responses(
        (status = 200, description = "Run stored", body = SavedJobRunResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 404, description = "Job not found", body = ErrorBody),
        (status = 422, description = "Validation error, or a run that cannot be (`bad_run`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_job_run(
    State(state): State<AppState>,
    WithRejection(Path((job, started_at)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<RecordJobRunParams>,
) -> Result<ApiResponse<SavedJobRunResponse>, WebError> {
    let input = params.into_input(job_code(job)?, &started_at)?;

    let run = state
        .features
        .job
        .record_job_run_use_case
        .execute(input, Utc::now())
        .await?;

    Ok(ApiResponse::ok(SavedJobRunResponse::from(&run)))
}

/// List the data jobs with how each stands now and how its last 14 days went
#[utoipa::path(
    get,
    path = "/v1/dashboard/jobs",
    tag = "jobs",
    responses(
        (status = 200, description = "Jobs retrieved successfully", body = JobsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs jobs:read", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_jobs(
    State(state): State<AppState>,
) -> Result<ApiResponse<JobsResponse>, WebError> {
    let statuses = state
        .features
        .job
        .view_jobs_use_case
        .execute(Utc::now())
        .await?;

    Ok(ApiResponse::ok(JobsResponse {
        jobs: statuses.iter().map(JobStatusResponse::from).collect(),
    }))
}
