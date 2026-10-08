use axum::extract::{Path, Query, State};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        RecordOutlookRunParams, RecordZoneOutlookParams, SavedOutlookRunResponse,
        SavedZoneOutlookResponse, SeasonOutlookQuery, SeasonOutlookResponse,
        ZoneOutlookHistoryResponse, ZoneOutlookQuery,
    },
    errors::WebError,
};

use crate::{
    infra::http::{ApiResponse, ErrorBody, ValidatedJson},
    shared::AppState,
};

/// Get the outlook for the next growing season, zone by zone
#[utoipa::path(
    get,
    path = "/v1/outlooks",
    tag = "outlooks",
    params(SeasonOutlookQuery),
    responses(
        (status = 200, description = "Outlook retrieved successfully", body = SeasonOutlookResponse),
        (status = 404, description = "No outlook issued for that season or month", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_season_outlook(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<SeasonOutlookQuery>, WebError>,
) -> Result<ApiResponse<SeasonOutlookResponse>, WebError> {
    let board = state
        .features
        .outlook
        .view_season_outlook_use_case
        .execute(query.into_input()?)
        .await?;

    Ok(ApiResponse::ok(SeasonOutlookResponse::from(&board)))
}

/// Get one zone's outlook at every issue of a season, oldest first
#[utoipa::path(
    get,
    path = "/v1/outlooks/zones/{zone_slug}",
    tag = "outlooks",
    params(
        ("zone_slug" = String, Path, description = "Zone slug, for example chamchamal"),
        ZoneOutlookQuery
    ),
    responses(
        (status = 200, description = "Zone outlook retrieved successfully", body = ZoneOutlookHistoryResponse),
        (status = 404, description = "No outlook issued yet", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_zone_outlook(
    State(state): State<AppState>,
    WithRejection(Path(zone_slug), _): WithRejection<Path<String>, WebError>,
    WithRejection(Query(query), _): WithRejection<Query<ZoneOutlookQuery>, WebError>,
) -> Result<ApiResponse<ZoneOutlookHistoryResponse>, WebError> {
    let (season, history) = state
        .features
        .outlook
        .view_zone_outlook_use_case
        .execute(query.into_input(zone_slug.clone())?)
        .await?;

    Ok(ApiResponse::ok(ZoneOutlookHistoryResponse {
        zone_slug,
        season: (&season).into(),
        issues: history.iter().map(Into::into).collect(),
    }))
}

/// Store one zone's outlook for one issue, replacing an earlier one
#[utoipa::path(
    put,
    path = "/v1/ingest/outlooks/{season}/{issued}/zones/{zone_slug}",
    tag = "outlooks",
    params(
        ("season" = String, Path, description = "Season, for example 2026-27"),
        ("issued" = String, Path, description = "Issue month, YYYY-MM"),
        ("zone_slug" = String, Path, description = "Zone slug, for example chamchamal"),
        ("X-Service-Key" = String, Header, description = "The data jobs' shared key")
    ),
    request_body = RecordZoneOutlookParams,
    responses(
        (status = 200, description = "Outlook stored", body = SavedZoneOutlookResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_zone_outlook(
    State(state): State<AppState>,
    WithRejection(Path((season, issued, zone_slug)), _): WithRejection<
        Path<(String, String, String)>,
        WebError,
    >,
    ValidatedJson(params): ValidatedJson<RecordZoneOutlookParams>,
) -> Result<ApiResponse<SavedZoneOutlookResponse>, WebError> {
    let input = params.into_input(season, issued, zone_slug)?;

    let outlook = state
        .features
        .outlook
        .record_zone_outlook_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(SavedZoneOutlookResponse::from(&outlook)))
}

/// Store the track record of the method behind one issue
#[utoipa::path(
    put,
    path = "/v1/ingest/outlooks/{season}/{issued}/run",
    tag = "outlooks",
    params(
        ("season" = String, Path, description = "Season, for example 2026-27"),
        ("issued" = String, Path, description = "Issue month, YYYY-MM"),
        ("X-Service-Key" = String, Header, description = "The data jobs' shared key")
    ),
    request_body = RecordOutlookRunParams,
    responses(
        (status = 200, description = "Track record stored", body = SavedOutlookRunResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Missing or wrong service key", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn put_outlook_run(
    State(state): State<AppState>,
    WithRejection(Path((season, issued)), _): WithRejection<Path<(String, String)>, WebError>,
    ValidatedJson(params): ValidatedJson<RecordOutlookRunParams>,
) -> Result<ApiResponse<SavedOutlookRunResponse>, WebError> {
    let input = params.into_input(season, issued)?;

    let run = state
        .features
        .outlook
        .record_outlook_run_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(SavedOutlookRunResponse::from(&run)))
}
