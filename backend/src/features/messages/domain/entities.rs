use chrono::{DateTime, Duration, Utc};
use getset::Getters;

use crate::{
    features::messages::domain::{
        IdempotencyKey, MessageError, MessageKind, MessageState, MessageText, Photo, PhotoType,
    },
    shared::Phone,
};

/// A stored photo without its bytes: what a message answer needs to link
/// to it.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct PhotoRef {
    id: i32,
    kind: PhotoType,
    size: u32,
}

impl PhotoRef {
    /// Reconstruct from persisted state.
    pub fn rehydrate(id: i32, kind: PhotoType, size: u32) -> Self {
        Self { id, kind, size }
    }
}

/// A stored photo with its bytes, read only when one photo is served.
#[derive(Clone, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct StoredPhoto {
    id: i32,
    message_id: i32,
    kind: PhotoType,
    bytes: Vec<u8>,
}

impl StoredPhoto {
    /// Reconstruct from persisted state.
    pub fn rehydrate(id: i32, message_id: i32, kind: PhotoType, bytes: Vec<u8>) -> Self {
        Self {
            id,
            message_id,
            kind,
            bytes,
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

impl std::fmt::Debug for StoredPhoto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredPhoto")
            .field("id", &self.id)
            .field("message_id", &self.message_id)
            .field("kind", &self.kind)
            .field("bytes", &self.bytes.len())
            .finish()
    }
}

/// What staff answered. A message has at most one: a second reply replaces
/// the first.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct Reply {
    text_ku: MessageText,
    text_en: Option<MessageText>,
    /// The staff member who answered. Staff see it; the farmer never does.
    replied_by: i32,
    replied_at: DateTime<Utc>,
}

impl Reply {
    pub fn new(
        text_ku: MessageText,
        text_en: Option<MessageText>,
        replied_by: i32,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            text_ku,
            text_en,
            replied_by,
            replied_at: now,
        }
    }

    /// Reconstruct from persisted state.
    pub fn rehydrate(
        text_ku: MessageText,
        text_en: Option<MessageText>,
        replied_by: i32,
        replied_at: DateTime<Utc>,
    ) -> Self {
        Self {
            text_ku,
            text_en,
            replied_by,
            replied_at,
        }
    }
}

/// One message a farmer sent to the Ministry.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Message {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    /// The farmer's id, not their phone: a message outlives its farmer, and
    /// must not keep a removed farmer's phone.
    farmer_id: i32,
    farm_id: Option<i32>,
    kind: MessageKind,
    text: MessageText,
    state: MessageState,
    reply: Option<Reply>,
    idempotency_key: Option<IdempotencyKey>,
    /// The photos already stored, without their bytes.
    photos: Vec<PhotoRef>,
    /// The photos of a message not yet stored. Empty once it is.
    new_photos: Vec<Photo>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Message {
    pub const MAX_PHOTOS: usize = 4;

    pub fn new(
        farmer_id: i32,
        farm_id: Option<i32>,
        kind: MessageKind,
        text: MessageText,
        photos: Vec<Photo>,
        idempotency_key: Option<IdempotencyKey>,
        now: DateTime<Utc>,
    ) -> Result<Self, MessageError> {
        if photos.len() > Self::MAX_PHOTOS {
            return Err(MessageError::TooManyPhotos(Self::MAX_PHOTOS));
        }

        Ok(Self {
            id: None,
            farmer_id,
            farm_id,
            kind,
            text,
            state: MessageState::New,
            reply: None,
            idempotency_key,
            photos: Vec::new(),
            new_photos: photos,
            created_at: now,
            updated_at: now,
        })
    }

    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        farmer_id: i32,
        farm_id: Option<i32>,
        kind: MessageKind,
        text: MessageText,
        state: MessageState,
        reply: Option<Reply>,
        idempotency_key: Option<IdempotencyKey>,
        photos: Vec<PhotoRef>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            farmer_id,
            farm_id,
            kind,
            text,
            state,
            reply,
            idempotency_key,
            photos,
            new_photos: Vec::new(),
            created_at,
            updated_at,
        }
    }

    pub fn was_sent_by(&self, farmer_id: i32) -> bool {
        self.farmer_id == farmer_id
    }

    /// Staff may move a message to any state by hand, except to `replied`
    /// when nothing was replied: the farmer would be told of an answer that
    /// does not exist.
    pub fn may_be_marked(&self, state: MessageState) -> Result<(), MessageError> {
        if state == MessageState::Replied && self.reply.is_none() {
            return Err(MessageError::NoReply);
        }

        Ok(())
    }
}

/// How many messages a farmer may send, so one phone cannot fill the inbox.
pub struct SendingLimit;

impl SendingLimit {
    pub const MAX_MESSAGES: u64 = 20;
    pub const WINDOW_HOURS: i64 = 24;

    /// Messages sent after this moment count against the limit.
    pub fn window_start(now: DateTime<Utc>) -> DateTime<Utc> {
        now - Duration::hours(Self::WINDOW_HOURS)
    }

    /// The seconds until the oldest counted message leaves the window and
    /// makes room for one more. Never zero: a caller told to wait must wait.
    pub fn retry_after_s(oldest_counted: DateTime<Utc>, now: DateTime<Utc>) -> u64 {
        let frees_at = oldest_counted + Duration::hours(Self::WINDOW_HOURS);

        u64::try_from((frees_at - now).num_seconds())
            .unwrap_or_default()
            .max(1)
    }
}

/// How many messages are in each state, for the inbox badge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct MessageCounts {
    new: u64,
    read: u64,
    replied: u64,
    closed: u64,
}

impl MessageCounts {
    /// A state with no messages is absent from a grouped count: it is zero.
    pub fn from_groups(groups: &[(MessageState, u64)]) -> Self {
        let mut counts = Self::default();

        for (state, count) in groups {
            match state {
                MessageState::New => counts.new += count,
                MessageState::Read => counts.read += count,
                MessageState::Replied => counts.replied += count,
                MessageState::Closed => counts.closed += count,
            }
        }

        counts
    }
}

/// Who sent a message, as the farmers feature knows them now.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct FarmerContact {
    id: i32,
    name: Option<String>,
    phone: Phone,
}

impl FarmerContact {
    pub fn rehydrate(id: i32, name: Option<String>, phone: Phone) -> Self {
        Self { id, name, phone }
    }
}

/// The farm a message is about, as the farms feature knows it now. The
/// place is `None` until farms carry one.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct FarmCard {
    id: i32,
    name: String,
    governorate: Option<String>,
    zone_slug: Option<String>,
}

impl FarmCard {
    pub fn rehydrate(
        id: i32,
        name: String,
        governorate: Option<String>,
        zone_slug: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            governorate,
            zone_slug,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 9, 12, 0, 0)
            .single()
            .expect("time")
    }

    fn text() -> MessageText {
        MessageText::new("The canal is dry".to_string()).expect("text")
    }

    fn a_photo() -> Photo {
        Photo::new(PhotoType::Jpeg, PhotoType::Jpeg.signature().to_vec()).expect("photo")
    }

    fn a_message(photos: Vec<Photo>) -> Result<Message, MessageError> {
        Message::new(3, Some(7), MessageKind::Report, text(), photos, None, now())
    }

    #[test]
    fn a_new_message_is_unsaved_new_and_unanswered() {
        let message = a_message(vec![a_photo()]).expect("message");

        assert_eq!(*message.id(), None);
        assert_eq!(*message.state(), MessageState::New);
        assert!(message.reply().is_none());
        assert_eq!(message.new_photos().len(), 1);
        assert!(message.photos().is_empty());
        assert_eq!(*message.created_at(), now());
    }

    #[test]
    fn a_message_carries_four_photos_at_most() {
        assert!(a_message(vec![a_photo(); Message::MAX_PHOTOS]).is_ok());
        assert!(matches!(
            a_message(vec![a_photo(); Message::MAX_PHOTOS + 1]),
            Err(MessageError::TooManyPhotos(4))
        ));
    }

    #[test]
    fn a_message_without_photos_or_a_farm_is_fine() {
        let message = Message::new(3, None, MessageKind::Other, text(), vec![], None, now());

        assert!(message.is_ok());
    }

    #[test]
    fn a_message_knows_who_sent_it() {
        let message = a_message(vec![]).expect("message");

        assert!(message.was_sent_by(3));
        assert!(!message.was_sent_by(4));
    }

    #[test]
    fn a_message_with_no_reply_cannot_be_marked_as_replied() {
        let message = a_message(vec![]).expect("message");

        assert!(matches!(
            message.may_be_marked(MessageState::Replied),
            Err(MessageError::NoReply)
        ));

        for state in [MessageState::New, MessageState::Read, MessageState::Closed] {
            assert!(message.may_be_marked(state).is_ok(), "{state:?}");
        }
    }

    #[test]
    fn a_message_that_was_answered_can_be_marked_as_anything() {
        let message = Message::rehydrate(
            1,
            3,
            None,
            MessageKind::Question,
            text(),
            MessageState::Closed,
            Some(Reply::new(text(), None, 9, now())),
            None,
            vec![],
            now(),
            now(),
        );

        for state in MessageState::ALL {
            assert!(message.may_be_marked(state).is_ok(), "{state:?}");
        }
    }

    #[test]
    fn the_window_is_the_last_24_hours() {
        assert_eq!(
            SendingLimit::window_start(now()),
            now() - Duration::hours(24)
        );
    }

    #[test]
    fn the_wait_is_until_the_oldest_counted_message_is_a_day_old() {
        let oldest = now() - Duration::hours(23);

        assert_eq!(SendingLimit::retry_after_s(oldest, now()), 3_600);
    }

    #[test]
    fn the_wait_is_never_zero_or_negative() {
        let oldest = now() - Duration::hours(24);

        assert_eq!(SendingLimit::retry_after_s(oldest, now()), 1);
        assert_eq!(
            SendingLimit::retry_after_s(oldest - Duration::hours(5), now()),
            1
        );
    }

    #[test]
    fn a_state_with_no_messages_counts_zero() {
        let counts =
            MessageCounts::from_groups(&[(MessageState::New, 4), (MessageState::Closed, 2)]);

        assert_eq!(*counts.new(), 4);
        assert_eq!(*counts.read(), 0);
        assert_eq!(*counts.replied(), 0);
        assert_eq!(*counts.closed(), 2);
    }

    #[test]
    fn a_stored_photo_prints_its_size_not_its_bytes() {
        let printed = format!(
            "{:?}",
            StoredPhoto::rehydrate(1, 2, PhotoType::Png, vec![7; 300])
        );

        assert!(printed.contains("300"), "{printed}");
        assert!(!printed.contains("7, 7"), "{printed}");
    }
}
