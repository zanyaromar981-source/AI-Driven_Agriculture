use async_trait::async_trait;
use chrono::NaiveDate;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QuerySelect,
    sea_query::{Expr, ExprTrait, OnConflict},
};

use crate::{
    app::AppError as GlobalAppError,
    features::insights::{
        app::{AppError, InsightRepository},
        domain::{FarmInsight, ReadingStamp},
        infra::persistence::postgres::entities::farm_insights,
    },
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "insight repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(Debug)]
pub struct InsightPostgresRepository {
    conn: DatabaseConnection,
}

impl InsightPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl InsightRepository for InsightPostgresRepository {
    async fn find_all_by_farm(&self, farm_id: i32) -> Result<Vec<FarmInsight>, AppError> {
        let models = farm_insights::Entity::find()
            .filter(farm_insights::Column::FarmId.eq(farm_id))
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(FarmInsight::try_from).collect()
    }

    async fn find_all_stamps(&self) -> Result<Vec<ReadingStamp>, AppError> {
        // The measures are the heavy part of a row and the job does not
        // need them to see which readings are missing.
        let rows: Vec<(i32, String, NaiveDate)> = farm_insights::Entity::find()
            .select_only()
            .column(farm_insights::Column::FarmId)
            .column(farm_insights::Column::Topic)
            .column(farm_insights::Column::AsOf)
            .into_tuple()
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        rows.into_iter().map(ReadingStamp::try_from).collect()
    }

    async fn upsert(&self, entity: &FarmInsight) -> Result<FarmInsight, AppError> {
        // Farm and topic together are the key, so the next push for a topic
        // replaces the reading in one statement.
        let model = farm_insights::Entity::insert(farm_insights::ActiveModel::from(entity))
            .on_conflict(
                OnConflict::columns([farm_insights::Column::FarmId, farm_insights::Column::Topic])
                    .update_columns([
                        farm_insights::Column::AsOf,
                        farm_insights::Column::Source,
                        farm_insights::Column::Confidence,
                        farm_insights::Column::SummaryEn,
                        farm_insights::Column::SummaryKu,
                        farm_insights::Column::Measures,
                        farm_insights::Column::UpdatedAt,
                    ])
                    // A slower copy of a job must not put an older reading
                    // back over a newer one.
                    .action_and_where(Expr::cust("excluded.as_of").gte(Expr::col((
                        farm_insights::Entity,
                        farm_insights::Column::AsOf,
                    ))))
                    .to_owned(),
            )
            .exec_with_returning(&self.conn)
            .await;

        match model {
            Ok(model) => FarmInsight::try_from(model),
            // Nothing was written because a newer reading is stored: answer
            // with that one.
            Err(DbErr::RecordNotInserted | DbErr::RecordNotFound(_)) => {
                let kept = farm_insights::Entity::find()
                    .filter(farm_insights::Column::FarmId.eq(*entity.farm_id()))
                    .filter(farm_insights::Column::Topic.eq(String::from(*entity.topic())))
                    .one(&self.conn)
                    .await
                    .map_err(database_error)?
                    .ok_or(GlobalAppError::NotFound)?;

                FarmInsight::try_from(kept)
            }
            Err(error) => Err(database_error(error)),
        }
    }
}
