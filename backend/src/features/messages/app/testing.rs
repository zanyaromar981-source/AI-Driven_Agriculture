use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};

use crate::{
    app::{Action, AuthContext, Pagination, Permission, Resource, StaffContext, User},
    features::messages::{
        app::{
            AppError, MessageFarmDirectory, MessageFilter, MessageRepository, SendOutcome,
            SenderDirectory,
        },
        domain::{
            FarmCard, FarmerContact, IdempotencyKey, Message, MessageCounts, MessageKind,
            MessageState, MessageText, Photo, PhotoRef, PhotoType, Reply, SearchText, StoredPhoto,
        },
    },
    shared::Phone,
};

pub const PHONE: &str = "+9647501234567";
pub const FARMER_ID: i32 = 3;
pub const FARM_ID: i32 = 7;
pub const STAFF_ID: i32 = 9;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    FarmerIdOf {
        phone: String,
    },
    ContactsOf {
        farmer_ids: Vec<i32>,
    },
    IdsMatching {
        text: String,
    },
    IsOwnedBy {
        farm_id: i32,
        phone: String,
    },
    CardsOf {
        farm_ids: Vec<i32>,
    },
    IdsInPlace {
        governorate: Option<String>,
        zone_slug: Option<String>,
    },
    Send {
        farmer_id: i32,
        photos: usize,
        max_messages: u64,
    },
    FindByIdempotencyKey {
        farmer_id: i32,
        key: String,
    },
    FindPageByFarmer {
        farmer_id: i32,
        page: u64,
    },
    FindPhoto {
        message_id: i32,
        photo_id: i32,
        farmer_id: Option<i32>,
    },
    FindPage {
        filter: MessageFilter,
        page: u64,
    },
    FindById {
        id: i32,
    },
    SetState {
        id: i32,
        state: MessageState,
    },
    Reply {
        id: i32,
        replied_by: i32,
    },
    CountByState,
    Delete {
        id: i32,
    },
}

#[derive(Debug, Default)]
struct Script {
    no_farmer: bool,
    owns_no_farm: bool,
    stored: Vec<Message>,
    photo: Option<StoredPhoto>,
    limit_reached_since: Option<DateTime<Utc>>,
    lose_the_race_to_send: bool,
    contacts: Vec<FarmerContact>,
    cards: Vec<FarmCard>,
    matching_farmer_ids: Vec<i32>,
    farm_ids_in_place: Vec<i32>,
    fail_with_database_error: bool,
}

/// One fake standing in for every port of the feature, so a test can read
/// the calls of a whole use case in the order they happened.
#[derive(Debug, Clone, Default)]
pub struct Fakes {
    script: Arc<Mutex<Script>>,
    calls: Arc<Mutex<Vec<Call>>>,
}

impl Fakes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_stored(self, message: Message) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .stored
            .push(message);
        self
    }

    pub fn with_photo(self, photo: StoredPhoto) -> Self {
        self.script.lock().expect("script lock").photo = Some(photo);
        self
    }

    /// The phone on the token has no farmer any more.
    pub fn without_farmer(self) -> Self {
        self.script.lock().expect("script lock").no_farmer = true;
        self
    }

    /// The farm a message names is not one of the farmer's.
    pub fn owning_no_farm(self) -> Self {
        self.script.lock().expect("script lock").owns_no_farm = true;
        self
    }

    /// The farmer has sent every message the limit allows; the oldest of
    /// them at `oldest`.
    pub fn at_the_limit_since(self, oldest: DateTime<Utc>) -> Self {
        self.script.lock().expect("script lock").limit_reached_since = Some(oldest);
        self
    }

    /// A copy of the same send stored its message between the lookup by key
    /// and the write.
    pub fn losing_the_race_to_send(self, winner: Message) -> Self {
        {
            let mut script = self.script.lock().expect("script lock");

            script.lose_the_race_to_send = true;
            script.stored.push(winner);
        }

        self
    }

    pub fn with_contact(self, contact: FarmerContact) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .contacts
            .push(contact);
        self
    }

    pub fn with_card(self, card: FarmCard) -> Self {
        self.script.lock().expect("script lock").cards.push(card);
        self
    }

    pub fn with_farmers_matching(self, farmer_ids: Vec<i32>) -> Self {
        self.script.lock().expect("script lock").matching_farmer_ids = farmer_ids;
        self
    }

    pub fn with_farms_in_place(self, farm_ids: Vec<i32>) -> Self {
        self.script.lock().expect("script lock").farm_ids_in_place = farm_ids;
        self
    }

    pub fn failing(self) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .fail_with_database_error = true;
        self
    }

    pub fn calls(&self) -> Vec<Call> {
        self.calls.lock().expect("calls lock").clone()
    }

    pub fn stored(&self) -> Vec<Message> {
        self.script.lock().expect("script lock").stored.clone()
    }

    fn record(&self, call: Call) {
        self.calls.lock().expect("calls lock").push(call);
    }

    fn guard(&self) -> Result<(), AppError> {
        if self
            .script
            .lock()
            .expect("script lock")
            .fail_with_database_error
        {
            return Err(crate::app::AppError::DatabaseError("fake".to_string()).into());
        }

        Ok(())
    }
}

#[async_trait]
impl SenderDirectory for Fakes {
    async fn farmer_id_of(&self, phone: &Phone) -> Result<Option<i32>, AppError> {
        self.record(Call::FarmerIdOf {
            phone: String::from(phone.as_str()),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok((!script.no_farmer).then_some(FARMER_ID))
    }

    async fn contacts_of(&self, farmer_ids: &[i32]) -> Result<Vec<FarmerContact>, AppError> {
        self.record(Call::ContactsOf {
            farmer_ids: farmer_ids.to_vec(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .contacts
            .iter()
            .filter(|contact| farmer_ids.contains(contact.id()))
            .cloned()
            .collect())
    }

    async fn ids_matching(&self, search: &SearchText) -> Result<Vec<i32>, AppError> {
        self.record(Call::IdsMatching {
            text: search.as_str().to_string(),
        });
        self.guard()?;

        Ok(self
            .script
            .lock()
            .expect("script lock")
            .matching_farmer_ids
            .clone())
    }
}

#[async_trait]
impl MessageFarmDirectory for Fakes {
    async fn is_owned_by(&self, farm_id: i32, phone: &Phone) -> Result<bool, AppError> {
        self.record(Call::IsOwnedBy {
            farm_id,
            phone: String::from(phone.as_str()),
        });
        self.guard()?;

        Ok(!self.script.lock().expect("script lock").owns_no_farm)
    }

    async fn cards_of(&self, farm_ids: &[i32]) -> Result<Vec<FarmCard>, AppError> {
        self.record(Call::CardsOf {
            farm_ids: farm_ids.to_vec(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .cards
            .iter()
            .filter(|card| farm_ids.contains(card.id()))
            .cloned()
            .collect())
    }

    async fn ids_in_place(
        &self,
        governorate: Option<&str>,
        zone_slug: Option<&str>,
    ) -> Result<Vec<i32>, AppError> {
        self.record(Call::IdsInPlace {
            governorate: governorate.map(str::to_string),
            zone_slug: zone_slug.map(str::to_string),
        });
        self.guard()?;

        Ok(self
            .script
            .lock()
            .expect("script lock")
            .farm_ids_in_place
            .clone())
    }
}

#[async_trait]
impl MessageRepository for Fakes {
    async fn send(
        &self,
        message: &Message,
        max_messages: u64,
        _counted_since: DateTime<Utc>,
    ) -> Result<SendOutcome, AppError> {
        self.record(Call::Send {
            farmer_id: *message.farmer_id(),
            photos: message.new_photos().len(),
            max_messages,
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        if script.lose_the_race_to_send
            && let Some(winner) = script.stored.first()
        {
            return Ok(SendOutcome::Repeated(winner.clone()));
        }

        if let Some(oldest_counted) = script.limit_reached_since {
            return Ok(SendOutcome::LimitReached { oldest_counted });
        }

        let stored = persisted(message, i32::try_from(script.stored.len()).unwrap_or(0) + 1);
        script.stored.push(stored.clone());

        Ok(SendOutcome::Stored(stored))
    }

    async fn find_by_idempotency_key(
        &self,
        farmer_id: i32,
        key: &IdempotencyKey,
    ) -> Result<Option<Message>, AppError> {
        self.record(Call::FindByIdempotencyKey {
            farmer_id,
            key: key.as_str().to_string(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        if script.lose_the_race_to_send {
            return Ok(None);
        }

        Ok(script
            .stored
            .iter()
            .find(|message| {
                message.was_sent_by(farmer_id) && message.idempotency_key().as_ref() == Some(key)
            })
            .cloned())
    }

    async fn find_page_by_farmer(
        &self,
        farmer_id: i32,
        pagination: &Pagination,
    ) -> Result<(Vec<Message>, u64), AppError> {
        self.record(Call::FindPageByFarmer {
            farmer_id,
            page: *pagination.page(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let messages: Vec<Message> = script
            .stored
            .iter()
            .filter(|message| message.was_sent_by(farmer_id))
            .cloned()
            .collect();
        let count = messages.len() as u64;

        Ok((messages, count))
    }

    async fn find_photo(
        &self,
        message_id: i32,
        photo_id: i32,
        farmer_id: Option<i32>,
    ) -> Result<Option<StoredPhoto>, AppError> {
        self.record(Call::FindPhoto {
            message_id,
            photo_id,
            farmer_id,
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let sent_by_them = |farmer_id: i32| {
            script
                .stored
                .iter()
                .any(|message| *message.id() == Some(message_id) && message.was_sent_by(farmer_id))
        };

        Ok(script
            .photo
            .clone()
            .filter(|photo| *photo.id() == photo_id && *photo.message_id() == message_id)
            .filter(|_| farmer_id.is_none_or(sent_by_them)))
    }

    async fn find_page(
        &self,
        filter: &MessageFilter,
        pagination: &Pagination,
    ) -> Result<(Vec<Message>, u64), AppError> {
        self.record(Call::FindPage {
            filter: filter.clone(),
            page: *pagination.page(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let messages: Vec<Message> = script
            .stored
            .iter()
            .filter(|message| filter.state.is_none_or(|state| *message.state() == state))
            .filter(|message| filter.kind.is_none_or(|kind| *message.kind() == kind))
            .cloned()
            .collect();
        let count = messages.len() as u64;

        Ok((messages, count))
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Message>, AppError> {
        self.record(Call::FindById { id });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .stored
            .iter()
            .find(|message| *message.id() == Some(id))
            .cloned())
    }

    async fn set_state(
        &self,
        id: i32,
        state: MessageState,
        now: DateTime<Utc>,
    ) -> Result<Option<Message>, AppError> {
        self.record(Call::SetState { id, state });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        let Some(stored) = script
            .stored
            .iter_mut()
            .find(|message| *message.id() == Some(id))
        else {
            return Ok(None);
        };

        if stored.may_be_marked(state).is_err() {
            return Ok(None);
        }

        *stored = rebuilt(stored, state, stored.reply().clone(), now);

        Ok(Some(stored.clone()))
    }

    async fn reply(&self, id: i32, reply: &Reply) -> Result<Option<Message>, AppError> {
        self.record(Call::Reply {
            id,
            replied_by: *reply.replied_by(),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        let Some(stored) = script
            .stored
            .iter_mut()
            .find(|message| *message.id() == Some(id))
        else {
            return Ok(None);
        };

        *stored = rebuilt(
            stored,
            MessageState::Replied,
            Some(reply.clone()),
            *reply.replied_at(),
        );

        Ok(Some(stored.clone()))
    }

    async fn count_by_state(&self) -> Result<MessageCounts, AppError> {
        self.record(Call::CountByState);
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let groups: Vec<(MessageState, u64)> = script
            .stored
            .iter()
            .map(|message| (*message.state(), 1))
            .collect();

        Ok(MessageCounts::from_groups(&groups))
    }

    async fn delete(&self, id: i32) -> Result<bool, AppError> {
        self.record(Call::Delete { id });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");
        let before = script.stored.len();

        script.stored.retain(|message| *message.id() != Some(id));

        Ok(script.stored.len() < before)
    }
}

fn persisted(message: &Message, id: i32) -> Message {
    let photos = message
        .new_photos()
        .iter()
        .zip(1i32..)
        .map(|(photo, photo_id)| {
            PhotoRef::rehydrate(
                photo_id,
                photo.kind(),
                u32::try_from(photo.bytes().len()).unwrap_or(u32::MAX),
            )
        })
        .collect();

    Message::rehydrate(
        id,
        *message.farmer_id(),
        *message.farm_id(),
        *message.kind(),
        message.text().clone(),
        *message.state(),
        message.reply().clone(),
        message.idempotency_key().clone(),
        photos,
        *message.created_at(),
        *message.updated_at(),
    )
}

fn rebuilt(
    message: &Message,
    state: MessageState,
    reply: Option<Reply>,
    now: DateTime<Utc>,
) -> Message {
    Message::rehydrate(
        message.id().unwrap_or_default(),
        *message.farmer_id(),
        *message.farm_id(),
        *message.kind(),
        message.text().clone(),
        state,
        reply,
        message.idempotency_key().clone(),
        message.photos().clone(),
        *message.created_at(),
        now,
    )
}

pub fn phone() -> Phone {
    Phone::new(PHONE.to_string()).expect("phone")
}

pub fn auth_context() -> AuthContext {
    AuthContext::new(User::new(phone()), "token".to_string())
}

pub fn staff_context() -> StaffContext {
    StaffContext::new(
        STAFF_ID,
        "officer@example.org".to_string(),
        Action::ALL
            .into_iter()
            .map(|action| Permission::new(Resource::Messages, action))
            .collect(),
    )
}

pub fn a_time() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 9, 8, 0, 0)
        .single()
        .expect("time")
}

pub fn text(value: &str) -> MessageText {
    MessageText::new(value.to_string()).expect("text")
}

pub fn a_key() -> IdempotencyKey {
    IdempotencyKey::new("send-1".to_string()).expect("key")
}

pub fn a_photo() -> Photo {
    Photo::new(PhotoType::Jpeg, PhotoType::Jpeg.signature().to_vec()).expect("photo")
}

/// A stored message of the farmer `FARMER_ID` about the farm `FARM_ID`,
/// sent with the key of `a_key()` and carrying one photo with id 1.
pub fn a_message(id: i32) -> Message {
    a_message_of(id, FARMER_ID, MessageState::New)
}

pub fn a_message_of(id: i32, farmer_id: i32, state: MessageState) -> Message {
    Message::rehydrate(
        id,
        farmer_id,
        Some(FARM_ID),
        MessageKind::Report,
        text("The canal is dry"),
        state,
        None,
        Some(a_key()),
        vec![PhotoRef::rehydrate(1, PhotoType::Jpeg, 3)],
        a_time(),
        a_time(),
    )
}

pub fn a_stored_photo(message_id: i32) -> StoredPhoto {
    StoredPhoto::rehydrate(
        1,
        message_id,
        PhotoType::Jpeg,
        PhotoType::Jpeg.signature().to_vec(),
    )
}

pub fn a_contact() -> FarmerContact {
    FarmerContact::rehydrate(FARMER_ID, Some("Azad".to_string()), phone())
}

pub fn a_card() -> FarmCard {
    FarmCard::rehydrate(FARM_ID, "Upper field".to_string(), None, None)
}
