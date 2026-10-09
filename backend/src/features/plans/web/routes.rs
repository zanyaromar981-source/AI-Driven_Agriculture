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

pub fn routes() -> Router<AppState> {
    Router::new().route("/farms/{id}/plan", get(handlers::get_farm_plan))
}

/// Used by the plan job. Mounted under `/ingest`, behind the service key.
pub fn ingest_routes() -> Router<AppState> {
    Router::new()
        .route("/farms/plans/coverage", get(handlers::get_plan_coverage))
        .route("/farms/{id}/plan", put(handlers::put_farm_plan))
}

/// Used by staff to see the stored plan of any farm. Mounted under
/// `/dashboard`, behind the `staff_auth` layer. Read only: a plan is made
/// from a forecast by the job, never typed in by a person.
pub fn dashboard_routes() -> Router<AppState> {
    Router::new().route(
        "/farms/{id}/plan",
        get(handlers::get_dashboard_farm_plan)
            .route_layer(require!(Resource::Insights, Action::Read)),
    )
}
