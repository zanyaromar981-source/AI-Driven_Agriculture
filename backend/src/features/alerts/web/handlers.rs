use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        AlertDashboardListResponse, AlertDashboardResponse, AlertPendingPushResponse,
        AlertPendingPushesResponse, AlertsQueryDto, AlertsResponse, RecordAlertParams,
        RegisterDeviceParams,
    },
    errors::WebError,
};

use crate::{
    app::{AuthContext, StaffContext},
    features::alerts::{app::AppError, domain::PushToken},
    infra::http::{ApiResponse, ErrorBody, ValidatedJson},
    shared::AppState,
};

/// Ids travel as opaque strings. One that is not a number cannot name a
/// farm or an alert, so it is not found rather than a bad request.
fn numeric_id(raw: &str) -> Result<i32, WebError> {
    raw.parse().map_err(|_| WebError::not_found())
}

/// List the alerts of one of the farmer's own farms, newest day first
#[utoipa::path(
    get,
    path = "/v1/farms/{id}/alerts",
    tag = "alerts",
    params(("id" = String, Path, description = "Farm ID"), AlertsQueryDto),
    responses(
        (status = 200, description = "Alerts retrieved successfully", body = AlertsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 422, description = "`days` is not 1 to 90", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_farm_alerts(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
    WithRejection(Query(query), _): WithRejection<Query<AlertsQueryDto>, WebError>,
) -> Result<ApiResponse<AlertsResponse>, WebError> {
    let alerts = state
        .features
        .alert
        .list_farm_alerts_use_case
        .execute(&auth_context, numeric_id(&id)?, query.into_days()?)
        .await?;

    Ok(ApiResponse::ok(AlertsResponse::of_one_farm(&alerts)?))
}

/// List the alerts of all the farmer's farms, newest day first
#[utoipa::path(
    get,
    path = "/v1/alerts",
    tag = "alerts",
    params(AlertsQueryDto),
    responses(
        (status = 200, description = "Alerts retrieved successfully, each with its `farm_id`", body = AlertsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 422, description = "`days` is not 1 to 90", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_my_alerts(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Query(query), _): WithRejection<Query<AlertsQueryDto>, WebError>,
) -> Result<ApiResponse<AlertsResponse>, WebError> {
    let alerts = state
        .features
        .alert
        .list_my_alerts_use_case
        .execute(&auth_context, query.into_days()?)
        .await?;

    Ok(ApiResponse::ok(AlertsResponse::of_many_farms(&alerts)?))
}

/// Tick an alert as done
#[utoipa::path(
    post,
    path = "/v1/alerts/{id}/done",
    tag = "alerts",
    params(("id" = String, Path, description = "Alert ID")),
    responses(
        (status = 204, description = "Ticked, also when it already was"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Alert not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn mark_alert_done(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<StatusCode, WebError> {
    state
        .features
        .alert
        .mark_alert_done_use_case
        .execute(&auth_context, numeric_id(&id)?)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Register this phone for pushes, or change what it wants pushed
#[utoipa::path(
    post,
    path = "/v1/devices",
    tag = "alerts",
    request_body = RegisterDeviceParams,
    responses(
        (status = 204, description = "Registered, also when it already was"),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn register_device(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    ValidatedJson(params): ValidatedJson<RegisterDeviceParams>,
) -> Result<StatusCode, WebError> {
    state
        .features
        .alert
        .register_device_use_case
        .execute(&auth_context, params.into_input()?)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Stop pushes to one of the farmer's own phones
#[utoipa::path(
    delete,
    path = "/v1/devices/{push_token}",
    tag = "alerts",
    params(("push_token" = String, Path, description = "The token the phone registered")),
    responses(
        (status = 204, description = "Removed, also when it was not registered"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 422, description = "The token is longer than 4,096 characters", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_device(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path(push_token), _): WithRejection<Path<String>, WebError>,
) -> Result<StatusCode, WebError> {
    let push_token = PushToken::new(push_token).map_err(AppError::from)?;

    state
        .features
        .alert
        .remove_device_use_case
        .execute(&auth_context, push_token)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Store one alert of a farm under its key, replacing its wording
///
/// Sending the same key again never unticks the alert or marks it as not
/// pushed.
#[utoipa::path(
    put,
    path = "/v1/ingest/farms/{id}/alerts/{key}",
    tag = "alerts",
    params(
        ("id" = String, Path, description = "Farm ID"),
        ("key" = String, Path, description = "The job's name for the alert, unique within the farm, for example `frost:2026-10-11`"),
        ("X-Service-Key" = String, Header, description = "The ingest service key")
    ),
    request_body = RecordAlertParams,
    responses(
        (status = 200, description = "Alert stored", body = AlertDashboardResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_alert(
    State(state): State<AppState>,
    WithRejection(Path((id, key)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<RecordAlertParams>,
) -> Result<ApiResponse<AlertDashboardResponse>, WebError> {
    let input = params.into_input(numeric_id(&id)?, key)?;

    let alert = state
        .features
        .alert
        .record_alert_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(AlertDashboardResponse::try_from(&alert)?))
}

/// List the alarms a push sender may send now, with the phones to send to
///
/// At most one per farm, and none for a farm that already had a push today.
#[utoipa::path(
    get,
    path = "/v1/ingest/alerts/unpushed",
    tag = "alerts",
    params(("X-Service-Key" = String, Header, description = "The ingest service key")),
    responses(
        (status = 200, description = "Alarms retrieved successfully", body = AlertPendingPushesResponse),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_unpushed_alerts(
    State(state): State<AppState>,
) -> Result<ApiResponse<AlertPendingPushesResponse>, WebError> {
    let pending = state
        .features
        .alert
        .list_unpushed_alerts_use_case
        .execute()
        .await?;

    Ok(ApiResponse::ok(AlertPendingPushesResponse {
        alerts: pending
            .iter()
            .map(AlertPendingPushResponse::try_from)
            .collect::<Result<_, _>>()?,
    }))
}

/// Record that an alert was pushed
#[utoipa::path(
    post,
    path = "/v1/ingest/alerts/{id}/pushed",
    tag = "alerts",
    params(
        ("id" = String, Path, description = "Alert ID"),
        ("X-Service-Key" = String, Header, description = "The ingest service key")
    ),
    responses(
        (status = 204, description = "Marked, also when it already was"),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 404, description = "Alert not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn mark_alert_pushed(
    State(state): State<AppState>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<StatusCode, WebError> {
    state
        .features
        .alert
        .mark_alert_pushed_use_case
        .execute(numeric_id(&id)?)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Forget a push token the push service says is dead
#[utoipa::path(
    delete,
    path = "/v1/ingest/devices/{push_token}",
    tag = "alerts",
    params(
        ("push_token" = String, Path, description = "The dead token"),
        ("X-Service-Key" = String, Header, description = "The ingest service key")
    ),
    responses(
        (status = 204, description = "Removed, also when it was already gone"),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 422, description = "The token is longer than 4,096 characters", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn delete_dead_device(
    State(state): State<AppState>,
    WithRejection(Path(push_token), _): WithRejection<Path<String>, WebError>,
) -> Result<StatusCode, WebError> {
    let push_token = PushToken::new(push_token).map_err(AppError::from)?;

    state
        .features
        .alert
        .remove_dead_device_use_case
        .execute(push_token)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// List every stored alert of any farm, for staff
#[utoipa::path(
    get,
    path = "/v1/dashboard/farms/{id}/alerts",
    tag = "alerts",
    params(("id" = String, Path, description = "Farm ID")),
    responses(
        (status = 200, description = "Alerts retrieved successfully", body = AlertDashboardListResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs insights:read", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard_farm_alerts(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<ApiResponse<AlertDashboardListResponse>, WebError> {
    let alerts = state
        .features
        .alert
        .view_stored_farm_alerts_use_case
        .execute(&staff_context, numeric_id(&id)?)
        .await?;

    Ok(ApiResponse::ok(AlertDashboardListResponse::try_from(
        alerts.as_slice(),
    )?))
}

/// Delete one stored alert
#[utoipa::path(
    delete,
    path = "/v1/dashboard/alerts/{id}",
    tag = "alerts",
    params(("id" = String, Path, description = "Alert ID")),
    responses(
        (status = 204, description = "Delete was successful, also when the alert was already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs insights:delete", body = ErrorBody),
        (status = 404, description = "The alert id is not a number", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_dashboard_alert(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<StatusCode, WebError> {
    state
        .features
        .alert
        .remove_alert_use_case
        .execute(&staff_context, numeric_id(&id)?)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
