use axum::extract::{Path, Query, State};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        DamHistoryQuery, DamHistoryResponse, DamResponse, DamsResponse, RecordDamReadingParams,
        SavedDamReadingResponse,
    },
    errors::WebError,
};

use crate::{
    features::dams::domain::DamSlug,
    infra::http::{ApiResponse, ErrorBody, ValidatedJson},
    shared::AppState,
};

/// A slug that is not well formed cannot name a dam, so on a read it is not
/// found rather than a bad request.
fn dam_slug(raw: String) -> Result<DamSlug, WebError> {
    DamSlug::new(raw).map_err(|_| WebError::not_found())
}

/// List the dams with their latest reading and the one from a year before
#[utoipa::path(
    get,
    path = "/v1/dams",
    tag = "dams",
    responses(
        (status = 200, description = "Dams retrieved successfully", body = DamsResponse),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_dams(
    State(state): State<AppState>,
) -> Result<ApiResponse<DamsResponse>, WebError> {
    let statuses = state.features.dam.list_dams_use_case.execute().await?;

    Ok(ApiResponse::ok(DamsResponse {
        dams: statuses.iter().map(DamResponse::from).collect(),
    }))
}

/// Get one dam's readings over time, oldest first
#[utoipa::path(
    get,
    path = "/v1/dams/{slug}/history",
    tag = "dams",
    params(("slug" = String, Path, description = "Dam slug, for example dukan"), DamHistoryQuery),
    responses(
        (status = 200, description = "History retrieved successfully", body = DamHistoryResponse),
        (status = 404, description = "Dam not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_dam_history(
    State(state): State<AppState>,
    WithRejection(Path(slug), _): WithRejection<Path<String>, WebError>,
    WithRejection(Query(query), _): WithRejection<Query<DamHistoryQuery>, WebError>,
) -> Result<ApiResponse<DamHistoryResponse>, WebError> {
    let input = query.into_input(dam_slug(slug)?)?;

    let (dam, readings) = state
        .features
        .dam
        .view_dam_history_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(DamHistoryResponse::from((
        &dam,
        readings.as_slice(),
    ))))
}

/// Store one dam's reading for one day, replacing an earlier one for that day
#[utoipa::path(
    put,
    path = "/v1/ingest/dams/{slug}/readings/{day}",
    tag = "dams",
    params(
        ("slug" = String, Path, description = "Dam slug, for example dukan"),
        ("day" = String, Path, description = "Day of the reading, YYYY-MM-DD"),
        ("X-Service-Key" = String, Header, description = "The data jobs' shared key")
    ),
    request_body = RecordDamReadingParams,
    responses(
        (status = 200, description = "Reading stored", body = SavedDamReadingResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 404, description = "Dam not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_dam_reading(
    State(state): State<AppState>,
    WithRejection(Path((slug, day)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<RecordDamReadingParams>,
) -> Result<ApiResponse<SavedDamReadingResponse>, WebError> {
    let input = params.into_input(slug.clone(), day)?;

    let reading = state
        .features
        .dam
        .record_dam_reading_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(SavedDamReadingResponse {
        slug,
        reading: (&reading).into(),
    }))
}
