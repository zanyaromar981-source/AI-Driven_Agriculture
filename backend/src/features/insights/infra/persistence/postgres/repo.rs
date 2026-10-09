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
        domain::{FarmInsight, ReadingStamp, Topic},
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

    async fn create(&self, entity: &FarmInsight) -> Result<Option<FarmInsight>, AppError> {
        // The unique index on farm and topic decides, in this one statement,
        // which of two creates sent at the same moment wins.
        let model = farm_insights::Entity::insert(farm_insights::ActiveModel::from(entity))
            .on_conflict(
                OnConflict::columns([farm_insights::Column::FarmId, farm_insights::Column::Topic])
                    .do_nothing()
                    .to_owned(),
            )
            .exec_with_returning(&self.conn)
            .await;

        match model {
            Ok(model) => Ok(Some(FarmInsight::try_from(model)?)),
            Err(DbErr::RecordNotInserted | DbErr::RecordNotFound(_)) => Ok(None),
            Err(error) => Err(database_error(error)),
        }
    }

    async fn update(&self, entity: &FarmInsight) -> Result<Option<FarmInsight>, AppError> {
        // The entity's own mapping builds the JSON of the measures, so an
        // update stores them exactly as an insert does.
        let model = farm_insights::ActiveModel::from(entity);

        // No guard on `as_of` here, unlike the upsert: a person correcting a
        // reading overrides what is stored on purpose.
        let models = farm_insights::Entity::update_many()
            .col_expr(farm_insights::Column::AsOf, Expr::value(*entity.as_of()))
            .col_expr(
                farm_insights::Column::Source,
                Expr::value(String::from(entity.source())),
            )
            .col_expr(
                farm_insights::Column::Confidence,
                Expr::value(String::from(*entity.confidence())),
            )
            .col_expr(
                farm_insights::Column::SummaryEn,
                Expr::value(entity.summary_en().as_ref().map(String::from)),
            )
            .col_expr(
                farm_insights::Column::SummaryKu,
                Expr::value(entity.summary_ku().as_ref().map(String::from)),
            )
            .col_expr(
                farm_insights::Column::Measures,
                Expr::value(model.measures.unwrap()),
            )
            .col_expr(
                farm_insights::Column::UpdatedAt,
                Expr::value(entity.updated_at().naive_utc()),
            )
            .filter(farm_insights::Column::FarmId.eq(*entity.farm_id()))
            .filter(farm_insights::Column::Topic.eq(String::from(*entity.topic())))
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        models
            .into_iter()
            .next()
            .map(FarmInsight::try_from)
            .transpose()
    }

    async fn delete(&self, farm_id: i32, topic: Topic) -> Result<(), AppError> {
        farm_insights::Entity::delete_many()
            .filter(farm_insights::Column::FarmId.eq(farm_id))
            .filter(farm_insights::Column::Topic.eq(String::from(topic)))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(())
    }
}
