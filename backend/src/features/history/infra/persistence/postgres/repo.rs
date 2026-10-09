use std::collections::HashMap;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    TransactionTrait,
    sea_query::{Expr, ExprTrait, OnConflict},
};

use crate::{
    app::AppError as GlobalAppError,
    features::history::{
        app::{AppError, HistoryRepository},
        domain::{HistoryWindow, Metric, MetricCoverage, MonthlyPoint, Series, SeriesUpload},
        infra::persistence::postgres::{
            entities::{farm_history, farm_history_series},
            mappings::{StoredSpan, point_active_models},
        },
    },
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "history repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(Debug)]
pub struct HistoryPostgresRepository {
    conn: DatabaseConnection,
}

impl HistoryPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl HistoryRepository for HistoryPostgresRepository {
    async fn find_series(
        &self,
        farm_id: i32,
        metrics: &[Metric],
        window: &HistoryWindow,
    ) -> Result<Vec<Series>, AppError> {
        let names: Vec<String> = metrics.iter().map(|metric| (*metric).into()).collect();

        // At most seven metrics of 120 months each: the window is bounded
        // before it gets here.
        let rows = farm_history::Entity::find()
            .filter(farm_history::Column::FarmId.eq(farm_id))
            .filter(farm_history::Column::Metric.is_in(names.clone()))
            .filter(
                farm_history::Column::Month
                    .between(window.from().first_day(), window.to().first_day()),
            )
            .order_by_asc(farm_history::Column::Month)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let mut points: HashMap<String, Vec<MonthlyPoint>> = HashMap::new();

        for row in &rows {
            points
                .entry(row.metric.clone())
                .or_default()
                .push(MonthlyPoint::try_from(row)?);
        }

        let facts = farm_history_series::Entity::find()
            .filter(farm_history_series::Column::FarmId.eq(farm_id))
            .filter(farm_history_series::Column::Metric.is_in(names))
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        facts
            .into_iter()
            .filter_map(|model| {
                let points = points.remove(&model.metric)?;

                Some(Series::try_from((model, points)))
            })
            .collect()
    }

    async fn find_all_coverage(&self) -> Result<Vec<MetricCoverage>, AppError> {
        // The values are the heavy part and the job does not need them to
        // see what is missing: one row per farm and metric, not per month.
        let spans = farm_history::Entity::find()
            .select_only()
            .column(farm_history::Column::FarmId)
            .column(farm_history::Column::Metric)
            .column_as(farm_history::Column::Month.min(), "first_month")
            .column_as(farm_history::Column::Month.max(), "last_month")
            .column_as(farm_history::Column::Month.count(), "months")
            .group_by(farm_history::Column::FarmId)
            .group_by(farm_history::Column::Metric)
            .into_model::<StoredSpan>()
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let facts: Vec<(i32, String, NaiveDateTime)> = farm_history_series::Entity::find()
            .select_only()
            .column(farm_history_series::Column::FarmId)
            .column(farm_history_series::Column::Metric)
            .column(farm_history_series::Column::AsOf)
            .into_tuple()
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let mut as_of: HashMap<(i32, String), NaiveDateTime> = facts
            .into_iter()
            .map(|(farm_id, metric, as_of)| ((farm_id, metric), as_of))
            .collect();

        spans
            .into_iter()
            .filter_map(|span| {
                // Months and facts are written together; a span whose facts
                // were removed between the two reads is on its way out.
                let as_of = as_of.remove(&(span.farm_id, span.metric.clone()))?;

                Some(MetricCoverage::try_from((span, as_of)))
            })
            .collect()
    }

    async fn store(&self, upload: &SeriesUpload) -> Result<bool, AppError> {
        let transaction = self.conn.begin().await.map_err(database_error)?;

        // The series row goes first: its lock makes two pushes for the same
        // farm and metric take their turns, whichever months they carry.
        let facts =
            farm_history_series::Entity::insert(farm_history_series::ActiveModel::from(upload))
                .on_conflict(
                    OnConflict::columns([
                        farm_history_series::Column::FarmId,
                        farm_history_series::Column::Metric,
                    ])
                    .update_columns([
                        farm_history_series::Column::Unit,
                        farm_history_series::Column::Source,
                        farm_history_series::Column::AsOf,
                    ])
                    // A slower copy of the job must not put older facts
                    // back over newer ones.
                    .action_and_where(Expr::cust("excluded.as_of").gte(Expr::col((
                        farm_history_series::Entity,
                        farm_history_series::Column::AsOf,
                    ))))
                    .to_owned(),
                )
                .exec_without_returning(&transaction)
                .await;

        // No row written means newer facts are stored. They stay, and so
        // does every month under them: the values of the older push may
        // only fill the months that are missing.
        let newest = match facts {
            Ok(written) => written > 0,
            Err(DbErr::RecordNotInserted) => false,
            Err(error) => return Err(database_error(error)),
        };

        let mut on_conflict = OnConflict::columns([
            farm_history::Column::FarmId,
            farm_history::Column::Metric,
            farm_history::Column::Month,
        ]);

        if newest {
            on_conflict.update_column(farm_history::Column::Value);
        } else {
            on_conflict.do_nothing();
        }

        // A push holds at most 240 months of four values each, far below
        // the 65,535 parameters Postgres takes. Months not named here are
        // not touched.
        let months = farm_history::Entity::insert_many(point_active_models(upload))
            .on_conflict(on_conflict)
            .exec_without_returning(&transaction)
            .await;

        match months {
            // Every month of an older push was stored already.
            Ok(_) | Err(DbErr::RecordNotInserted) => {}
            Err(error) => return Err(database_error(error)),
        }

        transaction.commit().await.map_err(database_error)?;

        Ok(newest)
    }

    async fn delete_series(&self, farm_id: i32, metric: Metric) -> Result<(), AppError> {
        let transaction = self.conn.begin().await.map_err(database_error)?;

        // Same order as a push, series row first, so a clear and a push of
        // the same series wait for each other instead of deadlocking.
        farm_history_series::Entity::delete_many()
            .filter(farm_history_series::Column::FarmId.eq(farm_id))
            .filter(farm_history_series::Column::Metric.eq(String::from(metric)))
            .exec(&transaction)
            .await
            .map_err(database_error)?;

        farm_history::Entity::delete_many()
            .filter(farm_history::Column::FarmId.eq(farm_id))
            .filter(farm_history::Column::Metric.eq(String::from(metric)))
            .exec(&transaction)
            .await
            .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)
    }
}
