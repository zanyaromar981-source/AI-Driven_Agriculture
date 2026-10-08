use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
    sea_query::OnConflict,
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

    async fn create(&self, entity: &Farmer) -> Result<Farmer, AppError> {
        let model = farmers::ActiveModel::from(entity)
            .insert(&self.conn)
            .await
            .map_err(database_error)?;

        Farmer::try_from(model)
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

    async fn delete(&self, phone: &Phone) -> Result<(), AppError> {
        sign_in_challenges::Entity::delete_by_id(phone.as_str())
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(())
    }
}
