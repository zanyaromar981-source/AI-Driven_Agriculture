use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, ExprTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait, TryInsertResult,
    sea_query::{Expr, OnConflict},
};

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::farmers::{
        app::{AppError, FarmerRepository, SignInChallengeRepository},
        domain::{Farmer, FarmerName, Language, SignInChallenge},
        infra::persistence::postgres::entities::{farmers, sign_in_challenges},
    },
    shared::Phone,
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "farmer repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(Debug)]
pub struct FarmerPostgresRepository {
    conn: DatabaseConnection,
}

impl FarmerPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl FarmerRepository for FarmerPostgresRepository {
    async fn find_by_phone(&self, phone: &Phone) -> Result<Option<Farmer>, AppError> {
        let model = farmers::Entity::find()
            .filter(farmers::Column::Phone.eq(phone.as_str()))
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        model.map(Farmer::try_from).transpose()
    }

    async fn create_if_absent(&self, entity: &Farmer) -> Result<(), AppError> {
        // The phone is unique, so two sign-ins racing for a new phone cannot
        // both insert: the second does nothing.
        farmers::Entity::insert(farmers::ActiveModel::from(entity))
            .on_conflict_do_nothing_on([farmers::Column::Phone])
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn update(&self, entity: &Farmer) -> Result<Farmer, AppError> {
        if entity.id().is_none() {
            return Err(GlobalAppError::MissingValue(
                "Cannot update a farmer that has not been persisted".to_string(),
            )
            .into());
        }

        let model = farmers::ActiveModel::from(entity)
            .update(&self.conn)
            .await
            .map_err(database_error)?;

        Farmer::try_from(model)
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Farmer>, AppError> {
        let model = farmers::Entity::find_by_id(id)
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        model.map(Farmer::try_from).transpose()
    }

    async fn find_page(
        &self,
        phone: Option<&Phone>,
        pagination: &Pagination,
    ) -> Result<(Vec<Farmer>, u64), AppError> {
        let mut query = farmers::Entity::find();

        if let Some(phone) = phone {
            query = query.filter(farmers::Column::Phone.eq(phone.as_str()));
        }

        let count = query
            .clone()
            .count(&self.conn)
            .await
            .map_err(database_error)?;

        let models = query
            .order_by_desc(farmers::Column::CreatedAt)
            .order_by_desc(farmers::Column::Id)
            .offset(pagination.skip())
            .limit(*pagination.rows_per_page())
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let farmers = models
            .into_iter()
            .map(Farmer::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok((farmers, count))
    }

    async fn create(&self, entity: &Farmer) -> Result<Option<Farmer>, AppError> {
        // The unique index on the phone decides: a phone that already has a
        // farmer inserts nothing, so no row comes back. (The single-row
        // `exec_with_returning` reports that as an error, not as a conflict.)
        let inserted = farmers::Entity::insert(farmers::ActiveModel::from(entity))
            .on_conflict_do_nothing_on([farmers::Column::Phone])
            .exec_with_returning_many(&self.conn)
            .await
            .map_err(database_error)?;

        match inserted {
            TryInsertResult::Inserted(models) => {
                models.into_iter().next().map(Farmer::try_from).transpose()
            }
            TryInsertResult::Conflicted | TryInsertResult::Empty => Ok(None),
        }
    }

    async fn update_by_id(
        &self,
        id: i32,
        name: Option<&FarmerName>,
        language: Language,
        now: DateTime<Utc>,
    ) -> Result<Option<Farmer>, AppError> {
        let updated = farmers::Entity::update_many()
            .col_expr(
                farmers::Column::Name,
                Expr::value(name.map(|name| name.as_str().to_string())),
            )
            .col_expr(
                farmers::Column::Language,
                Expr::value(String::from(language)),
            )
            .col_expr(farmers::Column::UpdatedAt, Expr::value(now.naive_utc()))
            .filter(farmers::Column::Id.eq(id))
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        updated.into_iter().next().map(Farmer::try_from).transpose()
    }

    async fn delete_with_challenge(&self, id: i32) -> Result<bool, AppError> {
        let transaction = self.conn.begin().await.map_err(database_error)?;

        let deleted = farmers::Entity::delete_many()
            .filter(farmers::Column::Id.eq(id))
            .exec_with_returning(&transaction)
            .await
            .map_err(database_error)?;

        // A code still open for the phone would otherwise sign the removed
        // farmer straight back in.
        for farmer in &deleted {
            sign_in_challenges::Entity::delete_many()
                .filter(sign_in_challenges::Column::Phone.eq(farmer.phone.as_str()))
                .exec(&transaction)
                .await
                .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)?;

        Ok(!deleted.is_empty())
    }
}

#[derive(Debug)]
pub struct SignInChallengePostgresRepository {
    conn: DatabaseConnection,
}

impl SignInChallengePostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl SignInChallengeRepository for SignInChallengePostgresRepository {
    async fn find_by_phone(&self, phone: &Phone) -> Result<Option<SignInChallenge>, AppError> {
        let model = sign_in_challenges::Entity::find_by_id(phone.as_str())
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        model.map(SignInChallenge::try_from).transpose()
    }

    async fn save_if_due(
        &self,
        challenge: &SignInChallenge,
        sent_before: DateTime<Utc>,
    ) -> Result<bool, AppError> {
        // The phone is the key, so saving a new challenge replaces the old,
        // but only an old one that was sent long enough ago. One statement
        // checks and writes, so requests at the same moment cannot all pass.
        let stored =
            sign_in_challenges::Entity::insert(sign_in_challenges::ActiveModel::from(challenge))
                .on_conflict(
                    OnConflict::column(sign_in_challenges::Column::Phone)
                        .update_columns([
                            sign_in_challenges::Column::CodeHash,
                            sign_in_challenges::Column::Language,
                            sign_in_challenges::Column::Attempts,
                            sign_in_challenges::Column::SentAt,
                            sign_in_challenges::Column::ExpiresAt,
                            sign_in_challenges::Column::UsedAt,
                        ])
                        // A code that was already used does not hold the
                        // phone waiting; an unused one does until it is old
                        // enough.
                        .action_and_where(
                            Expr::col((
                                sign_in_challenges::Entity,
                                sign_in_challenges::Column::SentAt,
                            ))
                            .lte(sent_before.naive_utc())
                            .or(Expr::col((
                                sign_in_challenges::Entity,
                                sign_in_challenges::Column::UsedAt,
                            ))
                            .is_not_null()),
                        )
                        .to_owned(),
                )
                .exec_without_returning(&self.conn)
                .await
                .map_err(database_error)?;

        Ok(stored > 0)
    }

    async fn record_attempt(
        &self,
        phone: &Phone,
        max_attempts: u32,
    ) -> Result<Option<SignInChallenge>, AppError> {
        // One statement counts the attempt and enforces the limit, so
        // guesses sent at the same moment are counted one after another.
        let counted = sign_in_challenges::Entity::update_many()
            .col_expr(
                sign_in_challenges::Column::Attempts,
                Expr::col(sign_in_challenges::Column::Attempts).add(1),
            )
            .filter(sign_in_challenges::Column::Phone.eq(phone.as_str()))
            .filter(
                sign_in_challenges::Column::Attempts
                    .lt(i32::try_from(max_attempts).unwrap_or(i32::MAX)),
            )
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        counted
            .into_iter()
            .next()
            .map(SignInChallenge::try_from)
            .transpose()
    }

    async fn consume(
        &self,
        phone: &Phone,
        code_hash: &str,
        now: DateTime<Utc>,
        reusable_since: DateTime<Utc>,
    ) -> Result<bool, AppError> {
        // One statement decides: the first use stamps the time, a repeat
        // inside the window leaves the stamp alone, anything later matches
        // no row.
        let result = sign_in_challenges::Entity::update_many()
            .col_expr(
                sign_in_challenges::Column::UsedAt,
                Expr::cust_with_values("COALESCE(used_at, $1)", [now.naive_utc()]),
            )
            .filter(sign_in_challenges::Column::Phone.eq(phone.as_str()))
            .filter(sign_in_challenges::Column::CodeHash.eq(code_hash))
            .filter(
                Condition::any()
                    .add(sign_in_challenges::Column::UsedAt.is_null())
                    .add(sign_in_challenges::Column::UsedAt.gt(reusable_since.naive_utc())),
            )
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }
}
