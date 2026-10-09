use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbBackend, DbErr, EntityTrait,
    FromQueryResult, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Statement,
    TransactionTrait,
    sea_query::{Expr, extension::postgres::PgExpr},
};

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::messages::{
        app::{AppError, MessageFilter, MessageRepository, SendOutcome},
        domain::{
            IdempotencyKey, Message, MessageCounts, MessageState, PhotoRef, Reply, StoredPhoto,
        },
        infra::persistence::postgres::{
            entities::{message_photos, messages},
            mappings::{photo_active_model, photo_ref},
        },
    },
};

/// The first half of the advisory lock a send takes; the farmer's id is the
/// second half. It only has to differ from any other pair of numbers this
/// database is asked to lock.
const SEND_LOCK: i32 = 0x4d53_4753;

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "message repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

/// A photo row without its bytes.
#[derive(FromQueryResult)]
struct PhotoRow {
    id: i32,
    message_id: i32,
    content_type: String,
    size: i32,
}

#[derive(FromQueryResult)]
struct Counted {
    sent: i64,
    oldest: Option<chrono::NaiveDateTime>,
}

#[derive(FromQueryResult)]
struct StateCount {
    state: String,
    messages: i64,
}

#[derive(Debug)]
pub struct MessagePostgresRepository {
    conn: DatabaseConnection,
}

impl MessagePostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// The photos of each of the messages, oldest first, read without the
    /// `bytes` column: a list of messages must never load megabytes.
    async fn photo_refs<C: ConnectionTrait>(
        conn: &C,
        message_ids: Vec<i32>,
    ) -> Result<HashMap<i32, Vec<PhotoRef>>, AppError> {
        let mut photos: HashMap<i32, Vec<PhotoRef>> = HashMap::new();

        if message_ids.is_empty() {
            return Ok(photos);
        }

        let rows = message_photos::Entity::find()
            .select_only()
            .column(message_photos::Column::Id)
            .column(message_photos::Column::MessageId)
            .column(message_photos::Column::ContentType)
            .column(message_photos::Column::Size)
            .filter(message_photos::Column::MessageId.is_in(message_ids))
            .order_by_asc(message_photos::Column::Id)
            .into_model::<PhotoRow>()
            .all(conn)
            .await
            .map_err(database_error)?;

        for row in rows {
            photos.entry(row.message_id).or_default().push(photo_ref(
                row.id,
                &row.content_type,
                row.size,
            )?);
        }

        Ok(photos)
    }

    async fn load<C: ConnectionTrait>(
        conn: &C,
        model: messages::Model,
    ) -> Result<Message, AppError> {
        let photos = Self::photo_refs(conn, vec![model.id])
            .await?
            .remove(&model.id)
            .unwrap_or_default();

        Message::try_from((model, photos))
    }

    async fn load_all<C: ConnectionTrait>(
        conn: &C,
        models: Vec<messages::Model>,
    ) -> Result<Vec<Message>, AppError> {
        let mut photos =
            Self::photo_refs(conn, models.iter().map(|model| model.id).collect()).await?;

        models
            .into_iter()
            .map(|model| {
                let photos = photos.remove(&model.id).unwrap_or_default();

                Message::try_from((model, photos))
            })
            .collect()
    }

    async fn find_by_key<C: ConnectionTrait>(
        conn: &C,
        farmer_id: i32,
        key: &IdempotencyKey,
    ) -> Result<Option<Message>, AppError> {
        let model = messages::Entity::find()
            .filter(messages::Column::FarmerId.eq(farmer_id))
            .filter(messages::Column::IdempotencyKey.eq(key.as_str()))
            .one(conn)
            .await
            .map_err(database_error)?;

        match model {
            Some(model) => Ok(Some(Self::load(conn, model).await?)),
            None => Ok(None),
        }
    }
}

#[async_trait]
impl MessageRepository for MessagePostgresRepository {
    async fn send(
        &self,
        message: &Message,
        max_messages: u64,
        counted_since: DateTime<Utc>,
    ) -> Result<SendOutcome, AppError> {
        let farmer_id = *message.farmer_id();

        let transaction = self.conn.begin().await.map_err(database_error)?;

        // Sends by one farmer run one after another until the transaction
        // ends, so the count below is still true when the insert happens:
        // of many sends at the same moment, none can pass the limit. The
        // lock is per farmer, so farmers do not wait for each other.
        transaction
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT pg_advisory_xact_lock($1, $2)",
                [SEND_LOCK.into(), farmer_id.into()],
            ))
            .await
            .map_err(database_error)?;

        // Under the same lock a repeat of a key sees the first send's row.
        // The unique index on the farmer and the key is still what forbids
        // a second row.
        if let Some(key) = message.idempotency_key()
            && let Some(existing) = Self::find_by_key(&transaction, farmer_id, key).await?
        {
            return Ok(SendOutcome::Repeated(existing));
        }

        let counted = messages::Entity::find()
            .select_only()
            .column_as(messages::Column::Id.count(), "sent")
            .column_as(messages::Column::CreatedAt.min(), "oldest")
            .filter(messages::Column::FarmerId.eq(farmer_id))
            .filter(messages::Column::CreatedAt.gt(counted_since.naive_utc()))
            .into_model::<Counted>()
            .one(&transaction)
            .await
            .map_err(database_error)?;

        if let Some(Counted {
            sent,
            oldest: Some(oldest),
        }) = counted
            && u64::try_from(sent).unwrap_or_default() >= max_messages
        {
            return Ok(SendOutcome::LimitReached {
                oldest_counted: oldest.and_utc(),
            });
        }

        let inserted = messages::Entity::insert(messages::ActiveModel::from(message))
            .exec_with_returning(&transaction)
            .await
            .map_err(database_error)?;

        if !message.new_photos().is_empty() {
            // One statement for all the photos, and no `RETURNING`: the
            // bytes go to the database once and do not come back.
            message_photos::Entity::insert_many(
                message
                    .new_photos()
                    .iter()
                    .map(|photo| photo_active_model(inserted.id, photo, *message.created_at())),
            )
            .exec_without_returning(&transaction)
            .await
            .map_err(database_error)?;
        }

        let stored = Self::load(&transaction, inserted).await?;

        transaction.commit().await.map_err(database_error)?;

        Ok(SendOutcome::Stored(stored))
    }

    async fn find_by_idempotency_key(
        &self,
        farmer_id: i32,
        key: &IdempotencyKey,
    ) -> Result<Option<Message>, AppError> {
        Self::find_by_key(&self.conn, farmer_id, key).await
    }

    async fn find_page_by_farmer(
        &self,
        farmer_id: i32,
        pagination: &Pagination,
    ) -> Result<(Vec<Message>, u64), AppError> {
        let query = messages::Entity::find().filter(messages::Column::FarmerId.eq(farmer_id));

        let count = query
            .clone()
            .count(&self.conn)
            .await
            .map_err(database_error)?;

        let models = query
            .order_by_desc(messages::Column::CreatedAt)
            .order_by_desc(messages::Column::Id)
            .offset(pagination.skip())
            .limit(*pagination.rows_per_page())
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        Ok((Self::load_all(&self.conn, models).await?, count))
    }

    async fn find_photo(
        &self,
        message_id: i32,
        photo_id: i32,
        farmer_id: Option<i32>,
    ) -> Result<Option<StoredPhoto>, AppError> {
        // Whose message it is comes first, so another farmer's photo is
        // never read at all.
        if let Some(farmer_id) = farmer_id {
            let theirs = messages::Entity::find()
                .filter(messages::Column::Id.eq(message_id))
                .filter(messages::Column::FarmerId.eq(farmer_id))
                .count(&self.conn)
                .await
                .map_err(database_error)?;

            if theirs == 0 {
                return Ok(None);
            }
        }

        let model = message_photos::Entity::find()
            .filter(message_photos::Column::Id.eq(photo_id))
            .filter(message_photos::Column::MessageId.eq(message_id))
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        model.map(StoredPhoto::try_from).transpose()
    }

    async fn find_page(
        &self,
        filter: &MessageFilter,
        pagination: &Pagination,
    ) -> Result<(Vec<Message>, u64), AppError> {
        let mut query = messages::Entity::find();

        if let Some(state) = filter.state {
            query = query.filter(messages::Column::State.eq(String::from(state)));
        }

        if let Some(kind) = filter.kind {
            query = query.filter(messages::Column::Kind.eq(String::from(kind)));
        }

        if let Some(farm_ids) = &filter.farm_ids {
            query = query.filter(messages::Column::FarmId.is_in(farm_ids.clone()));
        }

        if let Some(search) = &filter.search {
            // The pattern escapes `%` and `_` with a backslash, which is
            // the escape character Postgres uses when none is named.
            let mut found = Condition::any().add(
                Expr::col((messages::Entity, messages::Column::Text))
                    .ilike(search.text.like_pattern()),
            );

            if !search.farmer_ids.is_empty() {
                found = found.add(messages::Column::FarmerId.is_in(search.farmer_ids.clone()));
            }

            query = query.filter(found);
        }

        let count = query
            .clone()
            .count(&self.conn)
            .await
            .map_err(database_error)?;

        let models = query
            .order_by_desc(messages::Column::CreatedAt)
            .order_by_desc(messages::Column::Id)
            .offset(pagination.skip())
            .limit(*pagination.rows_per_page())
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        Ok((Self::load_all(&self.conn, models).await?, count))
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Message>, AppError> {
        let model = messages::Entity::find_by_id(id)
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        match model {
            Some(model) => Ok(Some(Self::load(&self.conn, model).await?)),
            None => Ok(None),
        }
    }

    async fn set_state(
        &self,
        id: i32,
        state: MessageState,
        now: DateTime<Utc>,
    ) -> Result<Option<Message>, AppError> {
        // One statement decides whether there is a message to mark and
        // whether it may be marked so. It writes the state only, so a reply
        // stored at the same moment is not undone.
        let mut update = messages::Entity::update_many()
            .col_expr(messages::Column::State, Expr::value(String::from(state)))
            .col_expr(messages::Column::UpdatedAt, Expr::value(now.naive_utc()))
            .filter(messages::Column::Id.eq(id));

        if state == MessageState::Replied {
            update = update.filter(messages::Column::ReplyTextKu.is_not_null());
        }

        let updated = update
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        match updated.into_iter().next() {
            Some(model) => Ok(Some(Self::load(&self.conn, model).await?)),
            None => Ok(None),
        }
    }

    async fn reply(&self, id: i32, reply: &Reply) -> Result<Option<Message>, AppError> {
        let updated = messages::Entity::update_many()
            .col_expr(
                messages::Column::ReplyTextKu,
                Expr::value(Some(String::from(reply.text_ku()))),
            )
            .col_expr(
                messages::Column::ReplyTextEn,
                Expr::value(reply.text_en().as_ref().map(String::from)),
            )
            .col_expr(
                messages::Column::RepliedBy,
                Expr::value(Some(*reply.replied_by())),
            )
            .col_expr(
                messages::Column::RepliedAt,
                Expr::value(Some(reply.replied_at().naive_utc())),
            )
            .col_expr(
                messages::Column::State,
                Expr::value(String::from(MessageState::Replied)),
            )
            .col_expr(
                messages::Column::UpdatedAt,
                Expr::value(reply.replied_at().naive_utc()),
            )
            .filter(messages::Column::Id.eq(id))
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        match updated.into_iter().next() {
            Some(model) => Ok(Some(Self::load(&self.conn, model).await?)),
            None => Ok(None),
        }
    }

    async fn count_by_state(&self) -> Result<MessageCounts, AppError> {
        let rows = messages::Entity::find()
            .select_only()
            .column(messages::Column::State)
            .column_as(messages::Column::Id.count(), "messages")
            .group_by(messages::Column::State)
            .into_model::<StateCount>()
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let groups = rows
            .into_iter()
            .map(|row| {
                Ok((
                    MessageState::try_from(row.state.as_str())?,
                    u64::try_from(row.messages).unwrap_or_default(),
                ))
            })
            .collect::<Result<Vec<_>, AppError>>()?;

        Ok(MessageCounts::from_groups(&groups))
    }

    async fn delete(&self, id: i32) -> Result<bool, AppError> {
        // The photos go with their message: the foreign key cascades.
        let result = messages::Entity::delete_many()
            .filter(messages::Column::Id.eq(id))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }
}
