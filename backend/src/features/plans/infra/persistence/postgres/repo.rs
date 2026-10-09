use async_trait::async_trait;
use chrono::NaiveDateTime;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QuerySelect,
    sea_query::{Expr, ExprTrait, OnConflict},
};

use crate::{
    app::AppError as GlobalAppError,
    features::plans::{
        app::{AppError, PlanRepository},
        domain::{FarmPlan, PlanStamp},
        infra::persistence::postgres::entities::farm_plans,
    },
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "plan repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(Debug)]
pub struct PlanPostgresRepository {
    conn: DatabaseConnection,
}

impl PlanPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl PlanRepository for PlanPostgresRepository {
    async fn find_by_farm(&self, farm_id: i32) -> Result<Option<FarmPlan>, AppError> {
        farm_plans::Entity::find()
            .filter(farm_plans::Column::FarmId.eq(farm_id))
            .one(&self.conn)
            .await
            .map_err(database_error)?
            .map(FarmPlan::try_from)
            .transpose()
    }

    async fn find_all_stamps(&self) -> Result<Vec<PlanStamp>, AppError> {
        // The job asks every ten minutes and needs only the times.
        let rows: Vec<(i32, NaiveDateTime)> = farm_plans::Entity::find()
            .select_only()
            .column(farm_plans::Column::FarmId)
            .column(farm_plans::Column::Issued)
            .into_tuple()
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(rows.into_iter().map(PlanStamp::from).collect())
    }

    async fn upsert(&self, entity: &FarmPlan) -> Result<FarmPlan, AppError> {
        // The farm is the key, so the next push replaces the plan in one
        // statement.
        let model = farm_plans::Entity::insert(farm_plans::ActiveModel::from(entity))
            .on_conflict(
                OnConflict::column(farm_plans::Column::FarmId)
                    .update_columns([
                        farm_plans::Column::FromDay,
                        farm_plans::Column::Issued,
                        farm_plans::Column::RainMm,
                        farm_plans::Column::Tmin,
                        farm_plans::Column::Tmax,
                        farm_plans::Column::Alerts,
                        farm_plans::Column::Decisions,
                        farm_plans::Column::Source,
                        farm_plans::Column::UpdatedAt,
                    ])
                    // A slower copy of the job must not put an older plan
                    // back over a newer one.
                    .action_and_where(
                        Expr::cust("excluded.issued")
                            .gte(Expr::col((farm_plans::Entity, farm_plans::Column::Issued))),
                    )
                    .to_owned(),
            )
            .exec_with_returning(&self.conn)
            .await;

        match model {
            Ok(model) => FarmPlan::try_from(model),
            // Nothing was written because a newer plan is stored: answer
            // with that one.
            Err(DbErr::RecordNotInserted | DbErr::RecordNotFound(_)) => self
                .find_by_farm(*entity.farm_id())
                .await?
                .ok_or_else(|| GlobalAppError::NotFound.into()),
            Err(error) => Err(database_error(error)),
        }
    }
}
