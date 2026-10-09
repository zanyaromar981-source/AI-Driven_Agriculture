use axum::{
    Extension,
    extract::{Multipart, Path, Query, State, multipart::Field},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{
        DashboardMessageResponse, DashboardMessagesQuery, DashboardMessagesResponse,
        DashboardOneMessageResponse, MessageCountsResponse, MessagePhotoPart, MessageResponse,
        MessageSendForm, MyMessagesResponse, OneMessageResponse, ReplyToMessageParams,
        SetMessageStateParams,
    },
    errors::WebError,
};

use crate::{
    app::{AuthContext, Pagination, StaffContext},
    features::messages::domain::{
        Message, MessageError, MessageText, Photo, PhotoType, StoredPhoto,
    },
    infra::http::{ApiResponse, ErrorBody, PaginationQueryDto, ValidatedJson},
    shared::AppState,
};

const IDEMPOTENCY_KEY: &str = "idempotency-key";

/// `kind` and `farm_id` are a few bytes in any honest form.
const MAX_SHORT_FIELD_BYTES: usize = 200;

/// A photo never changes once stored, so a browser may keep it for a day.
/// `private`: it was fetched with a token and is not for shared caches.
const PHOTO_CACHE_CONTROL: &str = "private, max-age=86400";

/// Ids travel as opaque strings. One that is not a number cannot name a
/// message or a photo, so it is not found rather than a bad request.
fn id_of(raw: &str) -> Result<i32, WebError> {
    raw.parse().map_err(|_| WebError::not_found())
}

/// The photo's bytes under the type they were checked against when stored.
fn photo_response(photo: StoredPhoto) -> Response {
    (
        [
            (header::CONTENT_TYPE, photo.kind().as_str()),
            (header::CACHE_CONTROL, PHOTO_CACHE_CONTROL),
            // The type above is the truth: a browser must not guess another.
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        photo.into_bytes(),
    )
        .into_response()
}

/// Send a message to the Ministry
///
/// Sent as `multipart/form-data`. A farmer may send 20 messages in any 24
/// hours. Repeat the same `Idempotency-Key` on a retry to get the message
/// already stored instead of a second one.
#[utoipa::path(
    post,
    path = "/v1/messages",
    tag = "messages",
    params(("Idempotency-Key" = Option<String>, Header, description = "Repeat it on a retry to get the message already stored")),
    request_body(content = MessageSendForm, content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "The stored message, also for a repeat of the same key", body = OneMessageResponse),
        (status = 400, description = "The form cannot be read", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "`farm_id` is not one of the farmer's farms", body = ErrorBody),
        (status = 422, description = "`bad_photo`, or `invalid` (kind, text)", body = ErrorBody),
        (status = 426, description = "`update_required`: the app is older than the oldest version allowed", body = ErrorBody),
        (status = 429, description = "`rate_limited`: 20 messages in 24 hours, with `retry_after_s`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn send_message(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    headers: HeaderMap,
    WithRejection(multipart, _): WithRejection<Multipart, WebError>,
) -> Result<ApiResponse<OneMessageResponse>, WebError> {
    let idempotency_key = headers
        .get(IDEMPOTENCY_KEY)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    let input = read_form(multipart).await?.into_input(idempotency_key)?;

    let message = state
        .features
        .message
        .send_message_use_case
        .execute(&auth_context, input)
        .await?;

    Ok(ApiResponse::created(OneMessageResponse::try_from(
        &message,
    )?))
}

/// Reads the form part by part and stops at the first part over its limit,
/// so a fifth photo or an oversized one is refused before it is read.
async fn read_form(mut multipart: Multipart) -> Result<MessageSendForm, WebError> {
    let mut form = MessageSendForm::default();

    while let Some(field) = multipart.next_field().await? {
        match field.name().unwrap_or_default() {
            "text" => {
                let text = read_text(field, MessageText::MAX_BYTES)
                    .await?
                    .ok_or(MessageError::TextTooLong(MessageText::MAX_LENGTH))?;

                form.text = Some(text);
            }
            "photos" | "photos[]" => {
                if form.photos.len() == Message::MAX_PHOTOS {
                    return Err(MessageError::TooManyPhotos(Message::MAX_PHOTOS).into());
                }

                let kind = PhotoType::from_declared(field.content_type().unwrap_or_default())?;
                let bytes = read_capped(field, Photo::MAX_BYTES)
                    .await?
                    .ok_or(MessageError::PhotoTooLarge(Photo::MAX_MB))?;

                form.photos.push(MessagePhotoPart { kind, bytes });
            }
            "kind" => form.kind = Some(read_short_text(field, "kind").await?),
            "farm_id" => form.farm_id = Some(read_short_text(field, "farm_id").await?),
            // Anything else is skipped without being kept.
            _ => {}
        }
    }

    Ok(form)
}

/// The part's bytes, or `None` as soon as they pass `max_bytes`.
async fn read_capped(mut field: Field<'_>, max_bytes: usize) -> Result<Option<Vec<u8>>, WebError> {
    let mut bytes = Vec::new();

    while let Some(chunk) = field.chunk().await? {
        if bytes.len() + chunk.len() > max_bytes {
            return Ok(None);
        }

        bytes.extend_from_slice(&chunk);
    }

    Ok(Some(bytes))
}

async fn read_text(field: Field<'_>, max_bytes: usize) -> Result<Option<String>, WebError> {
    let name = field.name().unwrap_or_default().to_string();

    read_capped(field, max_bytes)
        .await?
        .map(|bytes| {
            String::from_utf8(bytes)
                .map_err(|_| WebError::BadField(format!("{name} must be UTF-8 text")))
        })
        .transpose()
}

async fn read_short_text(field: Field<'_>, name: &str) -> Result<String, WebError> {
    read_text(field, MAX_SHORT_FIELD_BYTES)
        .await?
        .ok_or_else(|| WebError::BadField(format!("{name} is too long")))
}

/// List the farmer's own messages, newest first, with the replies
#[utoipa::path(
    get,
    path = "/v1/messages/mine",
    tag = "messages",
    params(PaginationQueryDto),
    responses(
        (status = 200, description = "Messages retrieved successfully", body = MyMessagesResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 426, description = "`update_required`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_my_messages(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Query(page), _): WithRejection<Query<PaginationQueryDto>, WebError>,
) -> Result<ApiResponse<MyMessagesResponse>, WebError> {
    let pagination = Pagination::from(&page);

    let (messages, count) = state
        .features
        .message
        .list_my_messages_use_case
        .execute(&auth_context, pagination)
        .await?;

    Ok(ApiResponse::ok(MyMessagesResponse {
        messages: messages
            .iter()
            .map(MessageResponse::try_from)
            .collect::<Result<Vec<_>, _>>()?,
        count,
        page: *pagination.page(),
        rows_per_page: *pagination.rows_per_page(),
    }))
}

/// Get one photo of one of the farmer's own messages
#[utoipa::path(
    get,
    path = "/v1/messages/{id}/photos/{photo_id}",
    tag = "messages",
    params(
        ("id" = String, Path, description = "Message ID"),
        ("photo_id" = String, Path, description = "Photo ID")
    ),
    responses(
        (status = 200, description = "The photo's bytes, as `image/jpeg` or `image/png`", content_type = "image/jpeg", body = Vec<u8>),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "No such photo among the farmer's own messages", body = ErrorBody),
        (status = 426, description = "`update_required`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_my_message_photo(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path((id, photo_id)), _): WithRejection<Path<(String, String)>, WebError>,
) -> Result<Response, WebError> {
    let photo = state
        .features
        .message
        .view_my_photo_use_case
        .execute(&auth_context, id_of(&id)?, id_of(&photo_id)?)
        .await?;

    Ok(photo_response(photo))
}

/// List every farmer's messages, newest first
#[utoipa::path(
    get,
    path = "/v1/dashboard/messages",
    tag = "messages",
    params(DashboardMessagesQuery, PaginationQueryDto),
    responses(
        (status = 200, description = "Messages retrieved successfully", body = DashboardMessagesResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs messages:read", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_messages(
    State(state): State<AppState>,
    WithRejection(Query(query), _): WithRejection<Query<DashboardMessagesQuery>, WebError>,
    WithRejection(Query(page), _): WithRejection<Query<PaginationQueryDto>, WebError>,
) -> Result<ApiResponse<DashboardMessagesResponse>, WebError> {
    let pagination = Pagination::from(&page);
    let input = query.into_input(pagination)?;

    let (records, count) = state
        .features
        .message
        .list_messages_use_case
        .execute(input)
        .await?;

    Ok(ApiResponse::ok(DashboardMessagesResponse {
        messages: records
            .iter()
            .map(DashboardMessageResponse::try_from)
            .collect::<Result<Vec<_>, _>>()?,
        count,
        page: *pagination.page(),
        rows_per_page: *pagination.rows_per_page(),
    }))
}

/// Count the messages in each state, for the inbox badge
#[utoipa::path(
    get,
    path = "/v1/dashboard/messages/counts",
    tag = "messages",
    responses(
        (status = 200, description = "Counts retrieved successfully", body = MessageCountsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs messages:read", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_message_counts(
    State(state): State<AppState>,
) -> Result<ApiResponse<MessageCountsResponse>, WebError> {
    let counts = state
        .features
        .message
        .count_messages_use_case
        .execute()
        .await?;

    Ok(ApiResponse::ok(MessageCountsResponse::from(&counts)))
}

/// Get one message with its photo links
#[utoipa::path(
    get,
    path = "/v1/dashboard/messages/{id}",
    tag = "messages",
    params(("id" = String, Path, description = "Message ID")),
    responses(
        (status = 200, description = "Message retrieved successfully", body = DashboardOneMessageResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs messages:read", body = ErrorBody),
        (status = 404, description = "Message not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_message(
    State(state): State<AppState>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<ApiResponse<DashboardOneMessageResponse>, WebError> {
    let record = state
        .features
        .message
        .view_message_use_case
        .execute(id_of(&id)?)
        .await?;

    Ok(ApiResponse::ok(DashboardOneMessageResponse::try_from(
        &record,
    )?))
}

/// Move a message to another state
///
/// Any state may follow any other, except that `replied` needs a reply.
#[utoipa::path(
    put,
    path = "/v1/dashboard/messages/{id}",
    tag = "messages",
    params(("id" = String, Path, description = "Message ID")),
    request_body = SetMessageStateParams,
    responses(
        (status = 200, description = "Message updated successfully", body = DashboardOneMessageResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs messages:update", body = ErrorBody),
        (status = 404, description = "Message not found", body = ErrorBody),
        (status = 422, description = "`no_reply`: a message with no reply cannot be marked `replied`", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_set_message_state(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<SetMessageStateParams>,
) -> Result<ApiResponse<DashboardOneMessageResponse>, WebError> {
    let record = state
        .features
        .message
        .set_message_state_use_case
        .execute(&staff_context, id_of(&id)?, params.state.into())
        .await?;

    Ok(ApiResponse::ok(DashboardOneMessageResponse::try_from(
        &record,
    )?))
}

/// Reply to a message
///
/// The message becomes `replied`, whatever state it was in. A second reply
/// replaces the first. The farmer reads the reply in the app.
#[utoipa::path(
    post,
    path = "/v1/dashboard/messages/{id}/reply",
    tag = "messages",
    params(("id" = String, Path, description = "Message ID")),
    request_body = ReplyToMessageParams,
    responses(
        (status = 200, description = "Reply stored successfully", body = DashboardOneMessageResponse),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs messages:update", body = ErrorBody),
        (status = 404, description = "Message not found", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_reply_to_message(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
    ValidatedJson(params): ValidatedJson<ReplyToMessageParams>,
) -> Result<ApiResponse<DashboardOneMessageResponse>, WebError> {
    let input = params.into_input()?;

    let record = state
        .features
        .message
        .reply_to_message_use_case
        .execute(&staff_context, id_of(&id)?, input)
        .await?;

    Ok(ApiResponse::ok(DashboardOneMessageResponse::try_from(
        &record,
    )?))
}

/// Delete a message with its photos
#[utoipa::path(
    delete,
    path = "/v1/dashboard/messages/{id}",
    tag = "messages",
    params(("id" = String, Path, description = "Message ID")),
    responses(
        (status = 204, description = "Delete was successful, also when the message was already gone"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs messages:delete", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_delete_message(
    State(state): State<AppState>,
    Extension(staff_context): Extension<StaffContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
) -> Result<StatusCode, WebError> {
    // An id that is not a number names nothing, and nothing is gone already.
    if let Ok(id) = id.parse::<i32>() {
        state
            .features
            .message
            .delete_message_use_case
            .execute(&staff_context, id)
            .await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Get one photo of any farmer's message
#[utoipa::path(
    get,
    path = "/v1/dashboard/messages/{id}/photos/{photo_id}",
    tag = "messages",
    params(
        ("id" = String, Path, description = "Message ID"),
        ("photo_id" = String, Path, description = "Photo ID")
    ),
    responses(
        (status = 200, description = "The photo's bytes, as `image/jpeg` or `image/png`", content_type = "image/jpeg", body = Vec<u8>),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Needs messages:read", body = ErrorBody),
        (status = 404, description = "Photo not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn dashboard_get_message_photo(
    State(state): State<AppState>,
    WithRejection(Path((id, photo_id)), _): WithRejection<Path<(String, String)>, WebError>,
) -> Result<Response, WebError> {
    let photo = state
        .features
        .message
        .view_photo_use_case
        .execute(id_of(&id)?, id_of(&photo_id)?)
        .await?;

    Ok(photo_response(photo))
}
