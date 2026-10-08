use axum::extract::{Path, Query, State};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{FireResponse, FiresQueryDto, FiresResponse, RecordFireParams},
    errors::WebError,
};

use crate::{
    infra::http::{ApiResponse, ErrorBody, ValidatedJson},
    shared::AppState,
};

/// List the fires detected in the last hours, with their totals
#[utoipa::path(
    get,
    path = "/v1/fires",
    tag = "fires",
    params(FiresQueryDto),
    responses(
        (status = 200, description = "Fires retrieved successfully", body = FiresResponse),
        (status = 400, description = "Invalid query", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_fires(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<FiresQueryDto>, WebError>,
) -> Result<ApiResponse<FiresResponse>, WebError> {
    let window = query.into_window()?;

    let (fires, summary) = state
        .features
        .fire
        .list_fires_use_case
        .execute(window)
        .await?;

    Ok(ApiResponse::ok(FiresResponse::try_from((
        window,
        fires.as_slice(),
        &summary,
    ))?))
}

/// Store one fire under the job's own id, replacing an earlier push of it
#[utoipa::path(
    put,
    path = "/v1/ingest/fires/{external_id}",
    tag = "fires",
    params(
        ("external_id" = String, Path, description = "The job's own key for the fire, 1 to 100 printable ASCII characters"),
        ("X-Service-Key" = String, Header, description = "The ingest service key")
    ),
    request_body = RecordFireParams,
    responses(
        (status = 200, description = "Fire stored", body = FireResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_fire(
    State(state): State<AppState>,
    WithRejection(Path(external_id), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<RecordFireParams>,
) -> Result<ApiResponse<FireResponse>, WebError> {
    let input = params.into_input(external_id)?;

    let fire = state
        .features
        .fire
        .record_fire_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(FireResponse::try_from(&fire)?))
}
