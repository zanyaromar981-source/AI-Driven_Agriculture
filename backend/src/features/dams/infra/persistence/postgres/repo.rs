use async_trait::async_trait;
use chrono::NaiveDate;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
    sea_query::OnConflict,
};

use crate::{
    app::AppError as GlobalAppError,
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
}
