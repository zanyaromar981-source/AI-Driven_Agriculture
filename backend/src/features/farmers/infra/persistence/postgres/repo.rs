use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ExprTrait, QueryFilter,
    sea_query::{Expr, OnConflict},
};

use crate::{
    app::AppError as GlobalAppError,
    features::farmers::{
        app::{AppError, FarmerRepository, SignInChallengeRepository},
        domain::{Farmer, SignInChallenge},
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

    async fn save(&self, challenge: &SignInChallenge) -> Result<(), AppError> {
        // The phone is the key, so saving a new challenge replaces the old.
        sign_in_challenges::Entity::insert(sign_in_challenges::ActiveModel::from(challenge))
            .on_conflict(
                OnConflict::column(sign_in_challenges::Column::Phone)
                    .update_columns([
                        sign_in_challenges::Column::CodeHash,
                        sign_in_challenges::Column::Language,
                        sign_in_challenges::Column::Attempts,
                        sign_in_challenges::Column::SentAt,
                        sign_in_challenges::Column::ExpiresAt,
                    ])
                    .to_owned(),
            )
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(())
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

    async fn consume(&self, phone: &Phone, code_hash: &str) -> Result<bool, AppError> {
        let result = sign_in_challenges::Entity::delete_many()
            .filter(sign_in_challenges::Column::Phone.eq(phone.as_str()))
            .filter(sign_in_challenges::Column::CodeHash.eq(code_hash))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }
}
