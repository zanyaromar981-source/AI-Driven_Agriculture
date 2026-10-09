use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    app::Pagination,
    features::messages::{
        app::AppError,
        domain::{
            IdempotencyKey, Message, MessageCounts, MessageKind, MessageState, Reply, SearchText,
            StoredPhoto,
        },
    },
};

/// What became of a message handed to `send`.
#[derive(Clone, Debug)]
pub enum SendOutcome {
    /// It was stored, with its photos.
    Stored(Message),
    /// The farmer already sent a message with this idempotency key: this is
    /// that message, and nothing was written.
    Repeated(Message),
    /// The farmer has sent as many messages as allowed since the window
    /// began, and nothing was written. `oldest_counted` is when the oldest
    /// of them was sent.
    LimitReached { oldest_counted: DateTime<Utc> },
}

/// What the text staff typed is looked for in: the message text itself, and
/// the farmers the farmers feature matched by name or phone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageSearch {
    pub text: SearchText,
    pub farmer_ids: Vec<i32>,
}

/// Which messages the staff inbox lists. Every part that is given must hold.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MessageFilter {
    pub state: Option<MessageState>,
    pub kind: Option<MessageKind>,
    /// Only messages about one of these farms.
    pub farm_ids: Option<Vec<i32>>,
    pub search: Option<MessageSearch>,
}

#[async_trait]
pub trait MessageRepository: Send + Sync + std::fmt::Debug {
    /// Stores the message and its photos in one transaction, unless the
    /// farmer already has `max_messages` messages sent after `counted_since`
    /// or already sent this idempotency key. Sends by one farmer wait for
    /// each other, so of several at the same moment none can pass the limit
    /// and a key is stored once. `message.id()` must be `None`.
    async fn send(
        &self,
        message: &Message,
        max_messages: u64,
        counted_since: DateTime<Utc>,
    ) -> Result<SendOutcome, AppError>;

    /// Returns the message an earlier send with the same key stored.
    async fn find_by_idempotency_key(
        &self,
        farmer_id: i32,
        key: &IdempotencyKey,
    ) -> Result<Option<Message>, AppError>;

    /// Returns one page of the farmer's own messages, newest first, with
    /// how many there are in all. No photo bytes are read.
    async fn find_page_by_farmer(
        &self,
        farmer_id: i32,
        pagination: &Pagination,
    ) -> Result<(Vec<Message>, u64), AppError>;

    /// Returns one photo of one message with its bytes. With `farmer_id`,
    /// only when that farmer sent the message.
    async fn find_photo(
        &self,
        message_id: i32,
        photo_id: i32,
        farmer_id: Option<i32>,
    ) -> Result<Option<StoredPhoto>, AppError>;

    // The methods below are not scoped to a farmer. They are for Ministry
    // staff on the dashboard; nothing a farmer's token reaches may call them.

    /// Returns one page of every farmer's messages, newest first, with how
    /// many match in all. No photo bytes are read.
    async fn find_page(
        &self,
        filter: &MessageFilter,
        pagination: &Pagination,
    ) -> Result<(Vec<Message>, u64), AppError>;

    async fn find_by_id(&self, id: i32) -> Result<Option<Message>, AppError>;

    /// Moves the message to `state` in one statement and returns it. Returns
    /// `None`, and writes nothing, when there is no such message or when
    /// `state` is `replied` and the message has no reply.
    async fn set_state(
        &self,
        id: i32,
        state: MessageState,
        now: DateTime<Utc>,
    ) -> Result<Option<Message>, AppError>;

    /// Stores the reply, replacing any earlier one, and moves the message to
    /// `replied`, in one statement. Returns `None` when there is no such
    /// message.
    async fn reply(&self, id: i32, reply: &Reply) -> Result<Option<Message>, AppError>;

    /// How many messages are in each state, from one grouped query.
    async fn count_by_state(&self) -> Result<MessageCounts, AppError>;

    /// Deletes the message and its photos. Returns whether there was one.
    async fn delete(&self, id: i32) -> Result<bool, AppError>;
}
