use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        DashboardWorkerResponse, DashboardWorkersQuery, DashboardWorkersResponse, MyWorkerResponse,
        OneWorkerResponse, PutWorkerParams, WorkerCardResponse, WorkerResponse, WorkersQuery,
        WorkersResponse,
    },
    errors::WebError,
};

use crate::{
    app::{AuthContext, Pagination, StaffContext},
    infra::http::{ApiResponse, ErrorBody, PaginationQueryDto, ValidatedJson},
    shared::AppState,
};

/// List the workers a farmer can call
///
/// Only available cards, and never the card of a blocked account. With
/// `lat` and `lon` the nearest come first, each with its `distance_km`;
/// cards without a point come last. Without, the newest update comes first.
#[utoipa::path(
    get,
    path = "/v1/workers",
    tag = "workers",
    params(WorkersQuery, PaginationQueryDto),
    responses(
        (status = 200, description = "Workers retrieved successfully", body = WorkersResponse),
        (status = 401, description = "Unauthorized: phone numbers are shown only to signed-in people", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_workers(
    State(state): State<AppState>,
    // Not read, but required: without a signed-in caller this handler must
    // not run at all, wherever the route is mounted.
    Extension(_auth_context): Extension<AuthContext>,
    WithRejection(Query(query), _): WithRejection<Query<WorkersQuery>, WebError>,
    WithRejection(Query(page), _): WithRejection<Query<PaginationQueryDto>, WebError>,
) -> Result<ApiResponse<WorkersResponse>, WebError> {
    let pagination = Pagination::from(&page);
    let input = query.into_input(pagination)?;

    let (listed, count) = state
        .features
        .worker
        .browse_workers_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(WorkersResponse {
        workers: listed
            .iter()
            .map(WorkerCardResponse::try_from)
            .collect::<Result<Vec<_>, _>>()?,
        count,
        page: *pagination.page(),
        rows_per_page: *pagination.rows_per_page(),
    }))
}

/// Get the caller's own worker card
#[utoipa::path(
    get,
    path = "/v1/workers/me",
    tag = "workers",
    responses(
        (status = 200, description = "The card, or `null` when the caller has none", body = MyWorkerResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_my_worker_card(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
) -> Result<ApiResponse<MyWorkerResponse>, WebError> {
    let card = state
        .features
        .worker
        .view_my_card_use_case
        .execute(&auth_context)
        .await?;

    Ok(ApiResponse::ok(MyWorkerResponse {
        worker: card.as_ref().map(WorkerResponse::try_from).transpose()?,
    }))
}

/// Put up or replace the caller's own worker card
///
/// One card per phone. The phone on the card is the phone the caller signed
/// in with: it cannot be sent. A second put replaces the first.
#[utoipa::path(
    put,
    path = "/v1/workers/me",
    tag = "workers",
    request_body = PutWorkerParams,
    responses(
        (status = 200, description = "The stored card", body = OneWorkerResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn put_my_worker_card(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    ValidatedJson(params): ValidatedJson<PutWorkerParams>,
) -> Result<ApiResponse<OneWorkerResponse>, WebError> {
    let details = params.into_details()?;

    let card = state
        .features
        .worker
        .put_my_card_use_case
        .execute(&auth_context, details)
        .await?;

    Ok(ApiResponse::ok(OneWorkerResponse {
        worker: WorkerResponse::try_from(&card)?,
    }))
}

/// Take down the caller's own worker card
#[utoipa::path(
    delete,
    path = "/v1/workers/me",
    tag = "workers",
    responses(
        (status = 204, description = "Delete was successful, also when there was no card"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_my_worker_card(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
) -> Result<StatusCode, WebError> {
    state
        .features
        .worker
        .remove_my_card_use_case
        .execute(&auth_context)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// List every worker card, also the ones that are not available
#[utoipa::path(
    get,
    path = "/v1/dashboard/workers",
    tag = "workers",
    params(DashboardWorkersQuery, PaginationQueryDto),
    responses(
        (status = 200, description = "Workers retrieved successfully", body = DashboardWorkersResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs farmers:read", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_workers(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<DashboardWorkersQuery>, WebError>,
    WithRejection(Query(page), _): WithRejection<Query<PaginationQueryDto>, WebError>,
) -> Result<ApiResponse<DashboardWorkersResponse>, WebError> {
    let pagination = Pagination::from(&page);
    let input = query.into_input(pagination)?;

    let (listed, count) = state
        .features
        .worker
        .list_all_workers_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(DashboardWorkersResponse {
        workers: listed
            .iter()
            .map(DashboardWorkerResponse::try_from)
            .collect::<Result<Vec<_>, _>>()?,
        count,
        page: *pagination.page(),
        rows_per_page: *pagination.rows_per_page(),
    }))
}

/// Delete a worker card
#[utoipa::path(
    delete,
    path = "/v1/dashboard/workers/{id}",
    tag = "workers",
    params(("id" = String, Path, description = "Worker card ID")),
    responses(
        (status = 204, description = "Delete was successful, also when the card was already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs farmers:delete", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_delete_worker(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<StatusCode, WebError> {
    // An id that is not a number names nothing, and nothing is gone already.
    if let Ok(id) = id.parse::<i32>() {
        state
            .features
            .worker
            .delete_worker_use_case
            .execute(&staff_context, id)
            .await?;
    }

    Ok(StatusCode::NO_CONTENT)
}
