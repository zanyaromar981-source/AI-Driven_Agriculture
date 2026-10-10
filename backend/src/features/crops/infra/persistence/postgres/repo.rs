use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder, TryInsertResult,
    sea_query::Expr,
};

use crate::{
    app::AppError as GlobalAppError,
    features::crops::{
        app::{AppError, CropRepository},
        domain::{Crop, CropCode, CropDetails},
        infra::persistence::postgres::entities::crops,
    },
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "crop repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(Debug)]
pub struct CropPostgresRepository {
    conn: DatabaseConnection,
}

impl CropPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl CropRepository for CropPostgresRepository {
    async fn find_all(&self, only_active: bool) -> Result<Vec<Crop>, AppError> {
        let mut query = crops::Entity::find();

        if only_active {
            query = query.filter(crops::Column::Active.eq(true));
        }

        let models = query
            .order_by_asc(crops::Column::SortOrder)
            .order_by_asc(crops::Column::Code)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(Crop::try_from).collect()
    }

    async fn find_by_code(&self, code: &CropCode) -> Result<Option<Crop>, AppError> {
        crops::Entity::find_by_id(code.as_str())
            .one(&self.conn)
            .await
            .map_err(database_error)?
            .map(Crop::try_from)
            .transpose()
    }

    async fn create(&self, crop: &Crop) -> Result<Option<Crop>, AppError> {
        // The code is the primary key, so of two creates sent at once one
        // inserts and the other does nothing.
        let inserted = crops::Entity::insert(crops::ActiveModel::from(crop))
            .on_conflict_do_nothing_on([crops::Column::Code])
            .exec_with_returning_many(&self.conn)
            .await
            .map_err(database_error)?;

        // A code that was taken returns no row.
        match inserted {
            TryInsertResult::Inserted(models) => {
                models.into_iter().next().map(Crop::try_from).transpose()
            }
            TryInsertResult::Conflicted | TryInsertResult::Empty => Ok(None),
        }
    }

    async fn update(
        &self,
        code: &CropCode,
        details: &CropDetails,
        now: DateTime<Utc>,
    ) -> Result<Option<Crop>, AppError> {
        // One statement: a crop deleted a moment ago matches no row, and
        // nothing brings it back.
        crops::Entity::update_many()
            .col_expr(
                crops::Column::NameEn,
                Expr::value(String::from(&details.name_en)),
            )
            .col_expr(
                crops::Column::NameKu,
                Expr::value(details.name_ku.as_ref().map(String::from)),
            )
            .col_expr(
                crops::Column::Color,
                Expr::value(String::from(&details.color)),
            )
            .col_expr(
                crops::Column::Category,
                Expr::value(String::from(details.category)),
            )
            .col_expr(
                crops::Column::Season,
                Expr::value(String::from(details.season)),
            )
            .col_expr(
                crops::Column::YieldKgPerDunam,
                Expr::value(details.yield_kg_per_dunam.map(|kg| kg.value())),
            )
            .col_expr(crops::Column::Grp, Expr::value(String::from(details.group)))
            .col_expr(crops::Column::Unit, Expr::value(String::from(details.unit)))
            .col_expr(crops::Column::Active, Expr::value(details.active))
            .col_expr(
                crops::Column::SortOrder,
                Expr::value(details.sort_order.value()),
            )
            .col_expr(crops::Column::UpdatedAt, Expr::value(now.naive_utc()))
            .filter(crops::Column::Code.eq(code.as_str()))
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?
            .into_iter()
            .next()
            .map(Crop::try_from)
            .transpose()
    }

    async fn delete(&self, code: &CropCode) -> Result<bool, AppError> {
        let result = crops::Entity::delete_many()
            .filter(crops::Column::Code.eq(code.as_str()))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }
}
