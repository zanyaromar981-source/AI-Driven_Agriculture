use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
    sea_query::{Expr, ExprTrait, OnConflict},
};

use crate::{
    app::AppError as GlobalAppError,
    features::fires::{
        app::{AppError, FireRepository},
        domain::Fire,
        infra::persistence::postgres::entities::fires,
    },
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "fire repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(Debug)]
pub struct FirePostgresRepository {
    conn: DatabaseConnection,
}

impl FirePostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl FireRepository for FirePostgresRepository {
    async fn find_detected_since(&self, since: DateTime<Utc>) -> Result<Vec<Fire>, AppError> {
        let models = fires::Entity::find()
            .filter(fires::Column::DetectedAt.gte(since.naive_utc()))
            .order_by_desc(fires::Column::DetectedAt)
            // Two detections of the same satellite pass share a time; the id
            // keeps their order the same from one request to the next.
            .order_by_desc(fires::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(Fire::try_from).collect()
    }

    async fn upsert(&self, entity: &Fire) -> Result<Fire, AppError> {
        // The external id is the key, so a second push of the same fire
        // replaces the first in one statement and two jobs cannot race.
        let model = fires::Entity::insert(fires::ActiveModel::from(entity))
            .on_conflict(
                OnConflict::column(fires::Column::ExternalId)
                    .update_columns([
                        fires::Column::Lat,
                        fires::Column::Lon,
                        fires::Column::ZoneSlug,
                        fires::Column::PlaceEn,
                        fires::Column::PlaceKu,
                        fires::Column::DetectedAt,
                        fires::Column::AreaHa,
                        fires::Column::WindKmh,
                        fires::Column::WindDirection,
                        fires::Column::Status,
                        fires::Column::FarmsWithin5km,
                        fires::Column::FarmersAlerted,
                        fires::Column::Source,
                        fires::Column::UpdatedAt,
                    ])
                    // A slower copy of a job must not put an older sighting
                    // back over a newer one.
                    .action_and_where(
                        Expr::cust("excluded.detected_at")
                            .gte(Expr::col((fires::Entity, fires::Column::DetectedAt))),
                    )
                    .to_owned(),
            )
            .exec_with_returning(&self.conn)
            .await;

        match model {
            Ok(model) => Fire::try_from(model),
            // Nothing was written because a newer sighting is stored: answer
            // with that one.
            Err(DbErr::RecordNotInserted | DbErr::RecordNotFound(_)) => {
                let kept = fires::Entity::find()
                    .filter(fires::Column::ExternalId.eq(entity.external_id().as_str()))
                    .one(&self.conn)
                    .await
                    .map_err(database_error)?
                    .ok_or(GlobalAppError::NotFound)?;

                Fire::try_from(kept)
            }
            Err(error) => Err(database_error(error)),
        }
    }
}
