use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::{NotSet, Set};

use crate::{
    app::AppError as GlobalAppError,
    features::messages::{
        app::AppError,
        domain::{
            IdempotencyKey, Message, MessageKind, MessageState, MessageText, Photo, PhotoRef,
            PhotoType, Reply, StoredPhoto,
        },
        infra::persistence::postgres::entities::{message_photos, messages},
    },
};

impl TryFrom<(messages::Model, Vec<PhotoRef>)> for Message {
    type Error = AppError;

    fn try_from((model, photos): (messages::Model, Vec<PhotoRef>)) -> Result<Self, Self::Error> {
        // A reply is its Sorani text; the other reply columns only mean
        // something beside it.
        let reply = match (model.reply_text_ku, model.replied_by, model.replied_at) {
            (Some(text_ku), Some(replied_by), Some(replied_at)) => Some(Reply::rehydrate(
                MessageText::new(text_ku)?,
                model.reply_text_en.map(MessageText::new).transpose()?,
                replied_by,
                replied_at.and_utc(),
            )),
            _ => None,
        };

        Ok(Message::rehydrate(
            model.id,
            model.farmer_id,
            model.farm_id,
            MessageKind::try_from(model.kind.as_str())?,
            MessageText::new(model.text)?,
            MessageState::try_from(model.state.as_str())?,
            reply,
            model.idempotency_key.map(IdempotencyKey::new).transpose()?,
            photos,
            model.created_at.and_utc(),
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&Message> for messages::ActiveModel {
    fn from(message: &Message) -> Self {
        let reply = message.reply().as_ref();

        messages::ActiveModel {
            id: match *message.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            farmer_id: Set(*message.farmer_id()),
            farm_id: Set(*message.farm_id()),
            kind: Set((*message.kind()).into()),
            text: Set(message.text().into()),
            state: Set((*message.state()).into()),
            reply_text_ku: Set(reply.map(|reply| reply.text_ku().into())),
            reply_text_en: Set(reply.and_then(|reply| reply.text_en().as_ref().map(Into::into))),
            replied_by: Set(reply.map(|reply| *reply.replied_by())),
            replied_at: Set(reply.map(|reply| reply.replied_at().naive_utc())),
            idempotency_key: Set(message.idempotency_key().as_ref().map(Into::into)),
            created_at: Set(message.created_at().naive_utc()),
            updated_at: Set(message.updated_at().naive_utc()),
        }
    }
}

pub fn photo_active_model(
    message_id: i32,
    photo: &Photo,
    now: DateTime<Utc>,
) -> message_photos::ActiveModel {
    message_photos::ActiveModel {
        id: NotSet,
        message_id: Set(message_id),
        content_type: Set(photo.kind().into()),
        bytes: Set(photo.bytes().to_vec()),
        // A photo is 4 MB at most, far inside an i32.
        size: Set(i32::try_from(photo.bytes().len()).unwrap_or(i32::MAX)),
        created_at: Set(now.naive_utc()),
    }
}

pub fn photo_ref(id: i32, content_type: &str, size: i32) -> Result<PhotoRef, AppError> {
    Ok(PhotoRef::rehydrate(
        id,
        PhotoType::try_from(content_type)?,
        u32::try_from(size).map_err(|_| {
            GlobalAppError::MissingValue("A stored photo has a negative size".to_string())
        })?,
    ))
}

impl TryFrom<message_photos::Model> for StoredPhoto {
    type Error = AppError;

    fn try_from(model: message_photos::Model) -> Result<Self, Self::Error> {
        Ok(StoredPhoto::rehydrate(
            model.id,
            model.message_id,
            PhotoType::try_from(model.content_type.as_str())?,
            model.bytes,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn at(hour: u32) -> chrono::NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 10, 9)
            .and_then(|day| day.and_hms_opt(hour, 0, 0))
            .expect("time")
    }

    fn a_model() -> messages::Model {
        messages::Model {
            id: 4,
            farmer_id: 3,
            farm_id: Some(7),
            kind: "complaint".to_string(),
            text: "No water for a week".to_string(),
            state: "replied".to_string(),
            reply_text_ku: Some("وەڵام".to_string()),
            reply_text_en: None,
            replied_by: Some(9),
            replied_at: Some(at(10)),
            idempotency_key: Some("send-1".to_string()),
            created_at: at(8),
            updated_at: at(10),
        }
    }

    #[test]
    fn a_stored_row_becomes_a_message_with_its_reply_and_photos() {
        let photos = vec![photo_ref(1, "image/png", 120).expect("ref")];

        let message = Message::try_from((a_model(), photos)).expect("message");
        let reply = message.reply().as_ref().expect("reply");

        assert_eq!(*message.id(), Some(4));
        assert_eq!(*message.kind(), MessageKind::Complaint);
        assert_eq!(*message.state(), MessageState::Replied);
        assert_eq!(*reply.replied_by(), 9);
        assert_eq!(reply.text_en(), &None);
        assert_eq!(message.photos().len(), 1);
        assert_eq!(*message.photos()[0].size(), 120);
    }

    #[test]
    fn a_row_with_no_reply_text_has_no_reply() {
        let model = messages::Model {
            reply_text_ku: None,
            replied_by: None,
            replied_at: None,
            state: "new".to_string(),
            ..a_model()
        };

        let message = Message::try_from((model, vec![])).expect("message");

        assert!(message.reply().is_none());
    }

    #[test]
    fn a_row_with_an_unknown_state_is_an_error_not_a_guess() {
        let model = messages::Model {
            state: "archived".to_string(),
            ..a_model()
        };

        assert!(Message::try_from((model, vec![])).is_err());
    }

    #[test]
    fn a_message_survives_the_trip_to_a_row_and_back() {
        let message = Message::try_from((a_model(), vec![])).expect("message");

        let active = messages::ActiveModel::from(&message);

        assert_eq!(active.id, Set(4));
        assert_eq!(active.kind, Set("complaint".to_string()));
        assert_eq!(active.reply_text_ku, Set(Some("وەڵام".to_string())));
        assert_eq!(active.replied_by, Set(Some(9)));
        assert_eq!(active.idempotency_key, Set(Some("send-1".to_string())));
    }

    #[test]
    fn a_photo_row_carries_its_type_and_its_size() {
        let photo = Photo::new(PhotoType::Png, PhotoType::Png.signature().to_vec()).expect("p");

        let active = photo_active_model(4, &photo, at(8).and_utc());

        assert_eq!(active.message_id, Set(4));
        assert_eq!(active.content_type, Set("image/png".to_string()));
        assert_eq!(active.size, Set(8));
    }

    #[test]
    fn a_photo_with_a_type_we_never_store_is_an_error() {
        assert!(photo_ref(1, "image/gif", 10).is_err());
        assert!(photo_ref(1, "image/png", -1).is_err());
    }
}
