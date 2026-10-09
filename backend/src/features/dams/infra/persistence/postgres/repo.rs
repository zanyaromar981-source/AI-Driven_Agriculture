use async_trait::async_trait;
use chrono::NaiveDate;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, TryInsertResult,
    sea_query::{Expr, OnConflict},
};

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::dams::{
        app::{AppError, DamRepository},
        domain::{Dam, DamReading, DamSlug},
        infra::persistence::postgres::entities::{dam_readings, dams},
    },
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "dam repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(Debug)]
pub struct DamPostgresRepository {
    conn: DatabaseConnection,
}

impl DamPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl DamRepository for DamPostgresRepository {
    async fn find_all(&self) -> Result<Vec<Dam>, AppError> {
        let models = dams::Entity::find()
            .order_by_asc(dams::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(Dam::try_from).collect()
    }

    async fn find_by_slug(&self, slug: &DamSlug) -> Result<Option<Dam>, AppError> {
        let model = dams::Entity::find()
            .filter(dams::Column::Slug.eq(slug.as_str()))
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        model.map(Dam::try_from).transpose()
    }

    async fn find_latest_reading(&self, dam_id: i32) -> Result<Option<DamReading>, AppError> {
        let model = dam_readings::Entity::find()
            .filter(dam_readings::Column::DamId.eq(dam_id))
            .order_by_desc(dam_readings::Column::Day)
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        model.map(DamReading::try_from).transpose()
    }

    async fn find_readings_between(
        &self,
        dam_id: i32,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<DamReading>, AppError> {
        let models = dam_readings::Entity::find()
            .filter(dam_readings::Column::DamId.eq(dam_id))
            .filter(dam_readings::Column::Day.between(from, to))
            .order_by_asc(dam_readings::Column::Day)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(DamReading::try_from).collect()
    }

    async fn upsert_reading(&self, reading: &DamReading) -> Result<DamReading, AppError> {
        // The dam and the day are the key, so a job that runs twice for the
        // same day replaces its earlier numbers instead of adding a row.
        let model = dam_readings::Entity::insert(dam_readings::ActiveModel::from(reading))
            .on_conflict(
                OnConflict::columns([dam_readings::Column::DamId, dam_readings::Column::Day])
                    .update_columns([
                        dam_readings::Column::PctFull,
                        dam_readings::Column::VolumeBnM3,
                        dam_readings::Column::LakeAreaKm2,
                        dam_readings::Column::FarmSupplyBnM3,
                        dam_readings::Column::Source,
                        dam_readings::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        DamReading::try_from(model)
    }

    async fn find_readings_page(
        &self,
        dam_id: i32,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        pagination: &Pagination,
    ) -> Result<(Vec<DamReading>, u64), AppError> {
        let mut query = dam_readings::Entity::find().filter(dam_readings::Column::DamId.eq(dam_id));

        if let Some(from) = from {
            query = query.filter(dam_readings::Column::Day.gte(from));
        }

        if let Some(to) = to {
            query = query.filter(dam_readings::Column::Day.lte(to));
        }

        let count = query
            .clone()
            .count(&self.conn)
            .await
            .map_err(database_error)?;

        let models = query
            .order_by_desc(dam_readings::Column::Day)
            .offset(pagination.skip())
            .limit(*pagination.rows_per_page())
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        Ok((
            models
                .into_iter()
                .map(DamReading::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            count,
        ))
    }

    async fn create_reading(&self, reading: &DamReading) -> Result<Option<DamReading>, AppError> {
        // One statement decides: the unique index on the dam and the day
        // lets exactly one of two racing creates insert, and the other gets
        // no row back.
        let inserted = dam_readings::Entity::insert(dam_readings::ActiveModel::from(reading))
            .on_conflict(
                OnConflict::columns([dam_readings::Column::DamId, dam_readings::Column::Day])
                    .do_nothing()
                    .to_owned(),
            )
            .try_insert()
            .exec_with_returning_many(&self.conn)
            .await
            .map_err(database_error)?;

        match inserted {
            // A conflict returns no row, which is an empty list, not an error.
            TryInsertResult::Inserted(mut models) => {
                models.pop().map(DamReading::try_from).transpose()
            }
            TryInsertResult::Conflicted | TryInsertResult::Empty => Ok(None),
        }
    }

    async fn update_reading(&self, reading: &DamReading) -> Result<Option<DamReading>, AppError> {
        let mut models = dam_readings::Entity::update_many()
            .col_expr(
                dam_readings::Column::PctFull,
                Expr::value(reading.pct_full().value()),
            )
            .col_expr(
                dam_readings::Column::VolumeBnM3,
                Expr::value(*reading.volume_bn_m3()),
            )
            .col_expr(
                dam_readings::Column::LakeAreaKm2,
                Expr::value(*reading.lake_area_km2()),
            )
            .col_expr(
                dam_readings::Column::FarmSupplyBnM3,
                Expr::value(*reading.farm_supply_bn_m3()),
            )
            .col_expr(
                dam_readings::Column::Source,
                Expr::value(reading.source().as_str()),
            )
            .col_expr(
                dam_readings::Column::UpdatedAt,
                Expr::value(reading.updated_at().naive_utc()),
            )
            .filter(dam_readings::Column::DamId.eq(*reading.dam_id()))
            .filter(dam_readings::Column::Day.eq(*reading.day()))
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        // The key is unique, so the update touched one row or none.
        models.pop().map(DamReading::try_from).transpose()
    }

    async fn delete_reading(&self, dam_id: i32, day: NaiveDate) -> Result<bool, AppError> {
        let result = dam_readings::Entity::delete_many()
            .filter(dam_readings::Column::DamId.eq(dam_id))
            .filter(dam_readings::Column::Day.eq(day))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }
}
