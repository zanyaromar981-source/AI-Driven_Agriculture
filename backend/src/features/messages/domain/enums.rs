use crate::{features::messages::domain::MessageError, shared::DomainError};

/// What the farmer says the message is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageKind {
    Question,
    Report,
    Complaint,
    Request,
    Other,
}

impl MessageKind {
    pub const ALL: [MessageKind; 5] = [
        MessageKind::Question,
        MessageKind::Report,
        MessageKind::Complaint,
        MessageKind::Request,
        MessageKind::Other,
    ];
}

impl From<MessageKind> for String {
    fn from(value: MessageKind) -> Self {
        match value {
            MessageKind::Question => "question".to_string(),
            MessageKind::Report => "report".to_string(),
            MessageKind::Complaint => "complaint".to_string(),
            MessageKind::Request => "request".to_string(),
            MessageKind::Other => "other".to_string(),
        }
    }
}

impl TryFrom<&str> for MessageKind {
    type Error = MessageError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "question" => Ok(MessageKind::Question),
            "report" => Ok(MessageKind::Report),
            "complaint" => Ok(MessageKind::Complaint),
            "request" => Ok(MessageKind::Request),
            "other" => Ok(MessageKind::Other),
            _ => Err(DomainError::InvalidValue(format!("Invalid message kind: {value}")).into()),
        }
    }
}

/// Where a message stands in the staff inbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MessageState {
    #[default]
    New,
    Read,
    Replied,
    Closed,
}

impl MessageState {
    pub const ALL: [MessageState; 4] = [
        MessageState::New,
        MessageState::Read,
        MessageState::Replied,
        MessageState::Closed,
    ];
}

impl From<MessageState> for String {
    fn from(value: MessageState) -> Self {
        match value {
            MessageState::New => "new".to_string(),
            MessageState::Read => "read".to_string(),
            MessageState::Replied => "replied".to_string(),
            MessageState::Closed => "closed".to_string(),
        }
    }
}

impl TryFrom<&str> for MessageState {
    type Error = MessageError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "new" => Ok(MessageState::New),
            "read" => Ok(MessageState::Read),
            "replied" => Ok(MessageState::Replied),
            "closed" => Ok(MessageState::Closed),
            _ => Err(DomainError::InvalidValue(format!("Invalid message state: {value}")).into()),
        }
    }
}

/// The image formats a message may carry. The string form is the media
/// type, which is also what the photo is served as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhotoType {
    Jpeg,
    Png,
}

impl PhotoType {
    pub const ALL: [PhotoType; 2] = [PhotoType::Jpeg, PhotoType::Png];

    /// The type a form part declares. Media types are case-insensitive and
    /// may carry parameters (`image/jpeg; name=a.jpg`), so only the bare
    /// type is compared.
    pub fn from_declared(media_type: &str) -> Result<Self, MessageError> {
        let bare = media_type
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();

        Self::try_from(bare.as_str()).map_err(|_| MessageError::PhotoNotAnImage)
    }

    /// The bytes every file of this type starts with.
    pub fn signature(self) -> &'static [u8] {
        match self {
            PhotoType::Jpeg => &[0xFF, 0xD8, 0xFF],
            PhotoType::Png => &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A],
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            PhotoType::Jpeg => "image/jpeg",
            PhotoType::Png => "image/png",
        }
    }
}

impl From<PhotoType> for String {
    fn from(value: PhotoType) -> Self {
        value.as_str().to_string()
    }
}

impl TryFrom<&str> for PhotoType {
    type Error = MessageError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "image/jpeg" => Ok(PhotoType::Jpeg),
            "image/png" => Ok(PhotoType::Png),
            _ => Err(DomainError::InvalidValue(format!("Invalid photo type: {value}")).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant must survive a trip to the database and back. A mismatch
    /// between the two directions corrupts rows silently rather than failing.
    #[test]
    fn every_kind_round_trips() {
        for kind in MessageKind::ALL {
            let stored = String::from(kind);

            assert_eq!(
                MessageKind::try_from(stored.as_str()).expect("kind"),
                kind,
                "{stored:?} did not round trip"
            );
        }
    }

    #[test]
    fn every_state_round_trips() {
        for state in MessageState::ALL {
            let stored = String::from(state);

            assert_eq!(
                MessageState::try_from(stored.as_str()).expect("state"),
                state,
                "{stored:?} did not round trip"
            );
        }
    }

    #[test]
    fn every_photo_type_round_trips() {
        for kind in PhotoType::ALL {
            let stored = String::from(kind);

            assert_eq!(
                PhotoType::try_from(stored.as_str()).expect("photo type"),
                kind,
                "{stored:?} did not round trip"
            );
        }
    }

    #[test]
    fn an_unknown_stored_value_is_rejected_rather_than_defaulted() {
        assert!(MessageKind::try_from("doctor").is_err());
        assert!(MessageKind::try_from("Question").is_err());
        assert!(MessageState::try_from("").is_err());
        assert!(MessageState::try_from("open").is_err());
    }

    #[test]
    fn a_message_starts_as_new() {
        assert_eq!(MessageState::default(), MessageState::New);
    }

    #[test]
    fn a_declared_type_is_read_whatever_its_case_or_parameters() {
        assert_eq!(
            PhotoType::from_declared("IMAGE/JPEG; name=a.jpg").expect("jpeg"),
            PhotoType::Jpeg
        );
        assert_eq!(
            PhotoType::from_declared(" image/png ").expect("png"),
            PhotoType::Png
        );
    }

    #[test]
    fn a_declared_type_that_is_not_jpeg_or_png_is_not_an_image() {
        for declared in ["image/gif", "application/pdf", "text/plain", ""] {
            assert!(
                matches!(
                    PhotoType::from_declared(declared),
                    Err(MessageError::PhotoNotAnImage)
                ),
                "{declared:?} must be refused"
            );
        }
    }
}
