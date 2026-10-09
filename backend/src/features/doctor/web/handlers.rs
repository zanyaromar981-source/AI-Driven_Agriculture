use axum::{
    Extension,
    extract::{Multipart, Path, State, multipart::Field},
};
use axum_extra::extract::WithRejection;

use super::{
    dtos::{DoctorAnswerResponse, DoctorAskForm, DoctorPhotoPart},
    errors::WebError,
};

use crate::{
    app::AuthContext,
    features::doctor::domain::{DoctorError, Enquiry, Photo, PhotoType, Question},
    infra::http::{ApiResponse, ErrorBody},
    shared::AppState,
};

/// `cell` and `lang` are a few bytes in any honest form.
const MAX_SHORT_FIELD_BYTES: usize = 200;

/// Farm ids travel as opaque strings. One that is not a number cannot name
/// a farm, so it is not found rather than a bad request.
fn farm_id(raw: &str) -> Result<i32, WebError> {
    raw.parse().map_err(|_| WebError::not_found())
}

/// Ask the Doctor about one of the farmer's farms
///
/// Sent as `multipart/form-data`. The question, photos and tapped cell go to
/// the local Doctor service together with the farm and what is known about
/// it; its answer comes back as it is once it has passed the rules. The
/// backend calls no AI itself. Nothing is stored: there is no `case_id` yet.
#[utoipa::path(
    post,
    path = "/v1/farms/{id}/ask",
    tag = "doctor",
    params(("id" = String, Path, description = "Farm ID")),
    request_body(content = DoctorAskForm, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "The Doctor's answer", body = DoctorAnswerResponse),
        (status = 400, description = "The form cannot be read", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Farm not found", body = ErrorBody),
        (status = 422, description = "`empty_question`, `bad_photo`, or `invalid` (question too long, unknown lang)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
        (status = 502, description = "`doctor_failed`: the Doctor service is down, timed out or failed", body = ErrorBody),
        (status = 503, description = "`doctor_not_ready`: the Doctor service cannot answer yet; try later", body = ErrorBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn ask_doctor(
    State(state): State<AppState>,
    Extension(auth_context): Extension<AuthContext>,
    WithRejection(Path(id), _): WithRejection<Path<String>, WebError>,
    WithRejection(multipart, _): WithRejection<Multipart, WebError>,
) -> Result<ApiResponse<DoctorAnswerResponse>, WebError> {
    let farm_id = farm_id(&id)?;

    let enquiry = read_form(multipart).await?.into_input()?;

    let answer = state
        .features
        .doctor
        .ask_doctor_use_case
        .execute(&auth_context, farm_id, enquiry)
        .await?;

    Ok(ApiResponse::ok(DoctorAnswerResponse::from(&answer)))
}

/// Reads the form part by part and stops at the first part over its limit,
/// so a seventh photo or an oversized one is refused before it is read.
async fn read_form(mut multipart: Multipart) -> Result<DoctorAskForm, WebError> {
    let mut form = DoctorAskForm::default();

    while let Some(field) = multipart.next_field().await? {
        match field.name().unwrap_or_default() {
            "question" => {
                let text = read_text(field, Question::MAX_BYTES)
                    .await?
                    .ok_or(DoctorError::QuestionTooLong(Question::MAX_LENGTH))?;

                form.question = Some(text);
            }
            "photos" | "photos[]" => {
                if form.photos.len() == Enquiry::MAX_PHOTOS {
                    return Err(DoctorError::TooManyPhotos(Enquiry::MAX_PHOTOS).into());
                }

                let kind = PhotoType::from_declared(field.content_type().unwrap_or_default())?;
                let bytes = read_capped(field, Photo::MAX_BYTES)
                    .await?
                    .ok_or(DoctorError::PhotoTooLarge(Photo::MAX_MB))?;

                form.photos.push(DoctorPhotoPart { kind, bytes });
            }
            "cell" => form.cell = Some(read_short_text(field, "cell").await?),
            "lang" => form.lang = Some(read_short_text(field, "lang").await?),
            // `voice` is reserved for later (BACKEND.md 2.5); anything else
            // is skipped the same way, without being kept.
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
