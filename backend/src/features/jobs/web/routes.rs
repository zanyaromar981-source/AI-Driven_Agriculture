use axum::{
    Router,
    routing::{get, put},
};

use crate::{
    app::{Action, Resource},
    require,
    shared::AppState,
};

use super::handlers;

/// Each data job reports its own runs here. Mounted under `/v1/ingest`,
/// behind the service key.
pub fn ingest_routes() -> Router<AppState> {
    Router::new().route("/jobs/{job}/runs/{started_at}", put(handlers::put_job_run))
}

/// Whether the jobs ran on time, for Ministry staff. Mounted under
/// `/v1/dashboard`, behind the `staff_auth` layer. Read only: the dashboard
/// cannot start a job, and jobs are neither made nor removed through it.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new().route(
        "/jobs",
        get(handlers::dashboard_get_jobs).route_layer(require!(Resource::Jobs, Action::Read)),
    )
}
