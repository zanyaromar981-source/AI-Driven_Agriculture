use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::messages::{
        app::{
            AppError, MessageRecord,
            use_cases::{ListMessagesInput, ReplyToMessageInput, SendMessageInput},
        },
        domain::{
            self, IdempotencyKey, Message, MessageCounts, MessageText, Photo, PhotoRef, PhotoType,
            SearchText,
        },
        web::errors::WebError,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum InboxMessageKind {
    Question,
    Report,
    Complaint,
    Request,
    Other,
}

impl From<InboxMessageKind> for domain::MessageKind {
    fn from(value: InboxMessageKind) -> Self {
        match value {
            InboxMessageKind::Question => domain::MessageKind::Question,
            InboxMessageKind::Report => domain::MessageKind::Report,
            InboxMessageKind::Complaint => domain::MessageKind::Complaint,
            InboxMessageKind::Request => domain::MessageKind::Request,
            InboxMessageKind::Other => domain::MessageKind::Other,
        }
    }
}

impl From<domain::MessageKind> for InboxMessageKind {
    fn from(value: domain::MessageKind) -> Self {
        match value {
            domain::MessageKind::Question => InboxMessageKind::Question,
            domain::MessageKind::Report => InboxMessageKind::Report,
            domain::MessageKind::Complaint => InboxMessageKind::Complaint,
            domain::MessageKind::Request => InboxMessageKind::Request,
            domain::MessageKind::Other => InboxMessageKind::Other,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum InboxMessageState {
    New,
    Read,
    Replied,
    Closed,
}

impl From<InboxMessageState> for domain::MessageState {
    fn from(value: InboxMessageState) -> Self {
        match value {
            InboxMessageState::New => domain::MessageState::New,
            InboxMessageState::Read => domain::MessageState::Read,
            InboxMessageState::Replied => domain::MessageState::Replied,
            InboxMessageState::Closed => domain::MessageState::Closed,
        }
    }
}

impl From<domain::MessageState> for InboxMessageState {
    fn from(value: domain::MessageState) -> Self {
        match value {
            domain::MessageState::New => InboxMessageState::New,
            domain::MessageState::Read => InboxMessageState::Read,
            domain::MessageState::Replied => InboxMessageState::Replied,
            domain::MessageState::Closed => InboxMessageState::Closed,
        }
    }
}

/// The form of `POST /v1/messages`, sent as `multipart/form-data`.
#[derive(Default, ToSchema)]
pub struct MessageSendForm {
    /// `question`, `report`, `complaint`, `request` or `other`.
    pub kind: Option<String>,
    /// What the farmer writes, 1 to 2000 characters.
    pub text: Option<String>,
    /// One of the farmer's own farms, when the message is about one.
    #[schema(required = false)]
    pub farm_id: Option<String>,
    /// 0 to 4 photos, each `image/jpeg` or `image/png` (the part's
    /// Content-Type) and 4 MB at most. One part per photo, named `photos`
    /// or `photos[]`.
    #[schema(value_type = Vec<String>, format = Binary, required = false)]
    pub photos: Vec<MessagePhotoPart>,
}

/// One photo part as it arrived, before its bytes are checked.
pub struct MessagePhotoPart {
    pub kind: PhotoType,
    pub bytes: Vec<u8>,
}

impl MessageSendForm {
    pub fn into_input(self, idempotency_key: Option<String>) -> Result<SendMessageInput, WebError> {
        let kind = domain::MessageKind::try_from(self.kind.unwrap_or_default().trim())?;
        let text = MessageText::new(self.text.unwrap_or_default())?;

        // Ids travel as opaque strings. One that is not a number cannot
        // name a farm, so it is not found.
        let farm_id = match self.farm_id.as_deref().map(str::trim) {
            None | Some("") => None,
            Some(raw) => Some(raw.parse::<i32>().map_err(|_| WebError::not_found())?),
        };

        let photos = self
            .photos
            .into_iter()
            .map(|part| Photo::new(part.kind, part.bytes))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(SendMessageInput {
            kind,
            text,
            farm_id,
            photos,
            idempotency_key: idempotency_key.map(IdempotencyKey::new).transpose()?,
        })
    }
}

/// A photo of a message: where to fetch it, never its bytes.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct MessagePhotoResponse {
    pub id: String,
    /// The path of the photo on this server, to be fetched with the same
    /// token.
    pub url: String,
    pub content_type: String,
    /// Bytes.
    pub size: u32,
}

impl MessagePhotoResponse {
    fn new(base: &str, message_id: i32, photo: &PhotoRef) -> Self {
        Self {
            id: photo.id().to_string(),
            url: format!("{base}/{message_id}/photos/{}", photo.id()),
            content_type: (*photo.kind()).into(),
            size: *photo.size(),
        }
    }
}

const FARMER_BASE: &str = "/v1/messages";
const DASHBOARD_BASE: &str = "/v1/dashboard/messages";

fn id_of(message: &Message) -> Result<i32, AppError> {
    message.id().ok_or_else(|| {
        AppError::GlobalAppError(GlobalAppError::MissingValue(
            "Message is missing its id".to_string(),
        ))
    })
}

/// The Ministry's answer as the farmer reads it. Who answered is not here.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct MessageReplyResponse {
    pub text_ku: String,
    pub text_en: Option<String>,
    pub replied_at: DateTime<Utc>,
}

/// A message as the farmer who sent it sees it.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct MessageResponse {
    pub id: String,
    pub kind: InboxMessageKind,
    pub text: String,
    pub farm_id: Option<String>,
    pub state: InboxMessageState,
    pub photos: Vec<MessagePhotoResponse>,
    /// `null` until staff answer.
    pub reply: Option<MessageReplyResponse>,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<&Message> for MessageResponse {
    type Error = AppError;

    fn try_from(message: &Message) -> Result<Self, Self::Error> {
        let id = id_of(message)?;

        Ok(Self {
            id: id.to_string(),
            kind: (*message.kind()).into(),
            text: message.text().into(),
            farm_id: message.farm_id().map(|farm_id| farm_id.to_string()),
            state: (*message.state()).into(),
            photos: message
                .photos()
                .iter()
                .map(|photo| MessagePhotoResponse::new(FARMER_BASE, id, photo))
                .collect(),
            reply: message.reply().as_ref().map(|reply| MessageReplyResponse {
                text_ku: reply.text_ku().into(),
                text_en: reply.text_en().as_ref().map(Into::into),
                replied_at: *reply.replied_at(),
            }),
            created_at: *message.created_at(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct OneMessageResponse {
    pub message: MessageResponse,
}

impl TryFrom<&Message> for OneMessageResponse {
    type Error = AppError;

    fn try_from(message: &Message) -> Result<Self, Self::Error> {
        Ok(Self {
            message: MessageResponse::try_from(message)?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct MyMessagesResponse {
    pub messages: Vec<MessageResponse>,
    /// How many messages the farmer has in all.
    pub count: u64,
    pub page: u64,
    pub rows_per_page: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardMessageReplyResponse {
    pub text_ku: String,
    pub text_en: Option<String>,
    /// The id of the staff member who answered.
    pub replied_by: String,
    pub replied_at: DateTime<Utc>,
}

/// A message as staff holding a `messages` permission see one, with the
/// farmer's name and phone.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardMessageResponse {
    pub id: String,
    pub kind: InboxMessageKind,
    pub text: String,
    pub state: InboxMessageState,
    pub farmer_id: String,
    /// `null` when the farmer set no name, or was removed since.
    pub farmer_name: Option<String>,
    /// `null` when the farmer was removed since.
    pub farmer_phone: Option<String>,
    pub farm_id: Option<String>,
    /// `null` when the message names no farm, or the farm was removed since.
    pub farm_name: Option<String>,
    /// Always `null` for now: farms carry no place yet.
    pub governorate: Option<String>,
    /// Always `null` for now: farms carry no place yet.
    pub zone_slug: Option<String>,
    pub photos: Vec<MessagePhotoResponse>,
    pub reply: Option<DashboardMessageReplyResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<&MessageRecord> for DashboardMessageResponse {
    type Error = AppError;

    fn try_from(record: &MessageRecord) -> Result<Self, Self::Error> {
        let message = &record.message;
        let id = id_of(message)?;
        let farmer = record.farmer.as_ref();
        let farm = record.farm.as_ref();

        Ok(Self {
            id: id.to_string(),
            kind: (*message.kind()).into(),
            text: message.text().into(),
            state: (*message.state()).into(),
            farmer_id: message.farmer_id().to_string(),
            farmer_name: farmer.and_then(|farmer| farmer.name().clone()),
            farmer_phone: farmer.map(|farmer| farmer.phone().as_str().to_string()),
            farm_id: message.farm_id().map(|farm_id| farm_id.to_string()),
            farm_name: farm.map(|farm| farm.name().clone()),
            governorate: farm.and_then(|farm| farm.governorate().clone()),
            zone_slug: farm.and_then(|farm| farm.zone_slug().clone()),
            photos: message
                .photos()
                .iter()
                .map(|photo| MessagePhotoResponse::new(DASHBOARD_BASE, id, photo))
                .collect(),
            reply: message
                .reply()
                .as_ref()
                .map(|reply| DashboardMessageReplyResponse {
                    text_ku: reply.text_ku().into(),
                    text_en: reply.text_en().as_ref().map(Into::into),
                    replied_by: reply.replied_by().to_string(),
                    replied_at: *reply.replied_at(),
                }),
            created_at: *message.created_at(),
            updated_at: *message.updated_at(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardOneMessageResponse {
    pub message: DashboardMessageResponse,
}

impl TryFrom<&MessageRecord> for DashboardOneMessageResponse {
    type Error = AppError;

    fn try_from(record: &MessageRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            message: DashboardMessageResponse::try_from(record)?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardMessagesResponse {
    pub messages: Vec<DashboardMessageResponse>,
    /// How many messages match in all.
    pub count: u64,
    pub page: u64,
    pub rows_per_page: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
pub struct MessageCountsResponse {
    pub new: u64,
    pub read: u64,
    pub replied: u64,
    pub closed: u64,
}

impl From<&MessageCounts> for MessageCountsResponse {
    fn from(counts: &MessageCounts) -> Self {
        Self {
            new: *counts.new(),
            read: *counts.read(),
            replied: *counts.replied(),
            closed: *counts.closed(),
        }
    }
}

fn given(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DashboardMessagesQuery {
    /// Only messages in this state: `new`, `read`, `replied` or `closed`.
    pub state: Option<String>,
    /// Only messages of this kind: `question`, `report`, `complaint`,
    /// `request` or `other`.
    pub kind: Option<String>,
    /// Only messages about a farm in this governorate. Farms carry no
    /// place yet, so for now this matches nothing.
    pub governorate: Option<String>,
    /// Only messages about a farm in this district (its slug). Farms carry
    /// no place yet, so for now this matches nothing.
    pub zone: Option<String>,
    /// Text to look for in the message, the farmer's name or the farmer's
    /// phone, whatever the case.
    pub q: Option<String>,
}

impl DashboardMessagesQuery {
    pub fn into_input(self, pagination: Pagination) -> Result<ListMessagesInput, AppError> {
        Ok(ListMessagesInput {
            state: given(self.state)
                .map(|state| domain::MessageState::try_from(state.as_str()))
                .transpose()?,
            kind: given(self.kind)
                .map(|kind| domain::MessageKind::try_from(kind.as_str()))
                .transpose()?,
            governorate: given(self.governorate),
            zone_slug: given(self.zone),
            search: given(self.q).map(SearchText::new).transpose()?,
            pagination,
        })
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SetMessageStateParams {
    pub state: InboxMessageState,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplyToMessageParams {
    /// The answer in Sorani, 1 to 2000 characters.
    pub text_ku: String,
    /// The same answer in English, 1 to 2000 characters. Optional.
    pub text_en: Option<String>,
}

impl ReplyToMessageParams {
    pub fn into_input(self) -> Result<ReplyToMessageInput, AppError> {
        Ok(ReplyToMessageInput {
            text_ku: MessageText::new(self.text_ku)?,
            text_en: self.text_en.map(MessageText::new).transpose()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::messages::{
        app::testing::{FARMER_ID, a_card, a_contact, a_message, a_time, text},
        domain::{MessageError, MessageState, Reply},
    };

    fn a_jpeg_part() -> MessagePhotoPart {
        MessagePhotoPart {
            kind: PhotoType::Jpeg,
            bytes: PhotoType::Jpeg.signature().to_vec(),
        }
    }

    fn a_form() -> MessageSendForm {
        MessageSendForm {
            kind: Some("report".to_string()),
            text: Some("The canal is dry".to_string()),
            farm_id: Some("7".to_string()),
            photos: vec![a_jpeg_part()],
        }
    }

    fn a_replied_message() -> Message {
        let message = a_message(4);

        Message::rehydrate(
            4,
            FARMER_ID,
            *message.farm_id(),
            *message.kind(),
            message.text().clone(),
            MessageState::Replied,
            Some(Reply::new(text("وەڵام"), Some(text("Answer")), 9, a_time())),
            None,
            message.photos().clone(),
            a_time(),
            a_time(),
        )
    }

    #[test]
    fn a_full_form_becomes_a_send() {
        let input = a_form()
            .into_input(Some("send-1".to_string()))
            .expect("input");

        assert_eq!(input.kind, domain::MessageKind::Report);
        assert_eq!(input.farm_id, Some(7));
        assert_eq!(input.photos.len(), 1);
        assert_eq!(
            input.idempotency_key.as_ref().map(IdempotencyKey::as_str),
            Some("send-1")
        );
    }

    #[test]
    fn a_form_without_a_farm_or_photos_is_fine() {
        for farm_id in [None, Some(String::new())] {
            let input = MessageSendForm {
                farm_id,
                photos: vec![],
                ..a_form()
            }
            .into_input(None)
            .expect("input");

            assert_eq!(input.farm_id, None);
        }
    }

    #[test]
    fn a_missing_or_unknown_kind_is_invalid() {
        for kind in [None, Some("doctor".to_string())] {
            let result = MessageSendForm { kind, ..a_form() }.into_input(None);

            assert!(matches!(
                result,
                Err(WebError::AppError(AppError::Message(
                    MessageError::DomainError(_)
                )))
            ));
        }
    }

    #[test]
    fn a_missing_text_is_invalid() {
        let result = MessageSendForm {
            text: None,
            ..a_form()
        }
        .into_input(None);

        assert!(matches!(result, Err(WebError::AppError(_))));
    }

    #[test]
    fn a_farm_id_that_is_not_a_number_is_not_found() {
        let result = MessageSendForm {
            farm_id: Some("abc".to_string()),
            ..a_form()
        }
        .into_input(None);

        assert!(matches!(
            result,
            Err(WebError::AppError(AppError::GlobalAppError(
                GlobalAppError::NotFound
            )))
        ));
    }

    #[test]
    fn a_photo_whose_bytes_are_not_its_declared_type_is_refused() {
        let result = MessageSendForm {
            photos: vec![MessagePhotoPart {
                kind: PhotoType::Png,
                bytes: b"not a picture".to_vec(),
            }],
            ..a_form()
        }
        .into_input(None);

        assert!(matches!(
            result,
            Err(WebError::AppError(AppError::Message(
                MessageError::PhotoNotAnImage
            )))
        ));
    }

    #[test]
    fn the_farmer_sees_the_reply_but_never_who_gave_it() {
        let response = MessageResponse::try_from(&a_replied_message()).expect("response");
        let json = serde_json::to_value(&response).expect("json");

        assert_eq!(json["state"], "replied");
        assert_eq!(json["reply"]["text_en"], "Answer");
        assert!(
            json["reply"].get("replied_by").is_none(),
            "the staff member's id must not reach the farmer"
        );
        assert!(json.get("farmer_phone").is_none());
    }

    #[test]
    fn a_photo_is_a_path_for_the_one_who_asks_never_bytes() {
        let farmer = MessageResponse::try_from(&a_message(4)).expect("response");
        let staff = DashboardMessageResponse::try_from(&MessageRecord {
            message: a_message(4),
            farmer: None,
            farm: None,
        })
        .expect("response");

        assert_eq!(farmer.photos[0].url, "/v1/messages/4/photos/1");
        assert_eq!(staff.photos[0].url, "/v1/dashboard/messages/4/photos/1");
        assert_eq!(farmer.photos[0].content_type, "image/jpeg");
    }

    #[test]
    fn staff_see_the_farmer_and_the_farm_with_a_null_place() {
        let response = DashboardMessageResponse::try_from(&MessageRecord {
            message: a_replied_message(),
            farmer: Some(a_contact()),
            farm: Some(a_card()),
        })
        .expect("response");
        let json = serde_json::to_value(&response).expect("json");

        assert_eq!(json["farmer_name"], "Azad");
        assert_eq!(json["farmer_phone"], "+9647501234567");
        assert_eq!(json["farm_name"], "Upper field");
        assert!(json["governorate"].is_null());
        assert!(json["zone_slug"].is_null());
        assert_eq!(json["reply"]["replied_by"], "9");
    }

    #[test]
    fn a_removed_farmer_shows_as_null_name_and_phone() {
        let response = DashboardMessageResponse::try_from(&MessageRecord {
            message: a_message(4),
            farmer: None,
            farm: None,
        })
        .expect("response");

        assert_eq!(response.farmer_id, FARMER_ID.to_string());
        assert_eq!(response.farmer_name, None);
        assert_eq!(response.farmer_phone, None);
        assert_eq!(response.farm_name, None);
    }

    #[test]
    fn blank_filters_are_no_filters() {
        let input = DashboardMessagesQuery {
            state: Some(String::new()),
            kind: Some(" ".to_string()),
            governorate: Some(String::new()),
            zone: None,
            q: Some("  ".to_string()),
        }
        .into_input(Pagination::new(1, 20))
        .expect("input");

        assert!(input.state.is_none());
        assert!(input.kind.is_none());
        assert!(input.governorate.is_none());
        assert!(input.search.is_none());
    }

    #[test]
    fn an_unknown_state_or_kind_filter_is_invalid() {
        let query = |state: Option<&str>, kind: Option<&str>| DashboardMessagesQuery {
            state: state.map(str::to_string),
            kind: kind.map(str::to_string),
            governorate: None,
            zone: None,
            q: None,
        };

        assert!(
            query(Some("open"), None)
                .into_input(Pagination::new(1, 20))
                .is_err()
        );
        assert!(
            query(None, Some("doctor"))
                .into_input(Pagination::new(1, 20))
                .is_err()
        );
    }

    #[test]
    fn every_kind_and_state_converts_both_ways() {
        for kind in domain::MessageKind::ALL {
            assert_eq!(
                domain::MessageKind::from(InboxMessageKind::from(kind)),
                kind
            );
        }

        for state in domain::MessageState::ALL {
            assert_eq!(
                domain::MessageState::from(InboxMessageState::from(state)),
                state
            );
        }
    }

    #[test]
    fn a_reply_needs_sorani_text_and_may_carry_english() {
        let both = ReplyToMessageParams {
            text_ku: "وەڵام".to_string(),
            text_en: Some("Answer".to_string()),
        }
        .into_input()
        .expect("input");

        assert_eq!(
            both.text_en.as_ref().map(MessageText::as_str),
            Some("Answer")
        );

        assert!(
            ReplyToMessageParams {
                text_ku: " ".to_string(),
                text_en: None
            }
            .into_input()
            .is_err()
        );
        assert!(
            ReplyToMessageParams {
                text_ku: "وەڵام".to_string(),
                text_en: Some(String::new())
            }
            .into_input()
            .is_err()
        );
    }
}
