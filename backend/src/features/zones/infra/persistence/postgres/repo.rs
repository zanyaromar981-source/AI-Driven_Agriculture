use async_trait::async_trait;
use chrono::NaiveDate;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    sea_query::OnConflict,
};

use crate::{
    app::AppError as GlobalAppError,
    features::zones::{
        app::{AppError, ZoneRepository},
        domain::{Month, SubZone, SubZoneReading, Zone, ZoneReading, ZoneSlug},
        infra::persistence::postgres::entities::{
            sub_zone_readings, sub_zones, zone_readings, zones,
        },
    },
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "zone repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(Debug)]
pub struct ZonePostgresRepository {
    conn: DatabaseConnection,
}

impl ZonePostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl ZoneRepository for ZonePostgresRepository {
    async fn find_all_zones(&self) -> Result<Vec<Zone>, AppError> {
        // The ids follow the seed order, which runs north to south.
        zones::Entity::find()
            .order_by_asc(zones::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(Zone::try_from)
            .collect()
    }

    async fn find_zone_by_slug(&self, slug: &ZoneSlug) -> Result<Option<Zone>, AppError> {
        let model = zones::Entity::find()
            .filter(zones::Column::Slug.eq(slug.as_str()))
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        model.map(Zone::try_from).transpose()
    }

    async fn find_sub_zones_by_zone(&self, zone_id: i32) -> Result<Vec<SubZone>, AppError> {
        sub_zones::Entity::find()
            .filter(sub_zones::Column::ZoneId.eq(zone_id))
            .order_by_asc(sub_zones::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(SubZone::try_from)
            .collect()
    }

    async fn find_sub_zone_by_slug(
        &self,
        zone_id: i32,
        slug: &ZoneSlug,
    ) -> Result<Option<SubZone>, AppError> {
        let model = sub_zones::Entity::find()
            .filter(sub_zones::Column::ZoneId.eq(zone_id))
            .filter(sub_zones::Column::Slug.eq(slug.as_str()))
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        model.map(SubZone::try_from).transpose()
    }

    async fn find_reading_months(&self) -> Result<Vec<Month>, AppError> {
        let first_days: Vec<NaiveDate> = zone_readings::Entity::find()
            .select_only()
            .column(zone_readings::Column::Month)
            .distinct()
            .order_by_asc(zone_readings::Column::Month)
            .into_tuple()
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(first_days.into_iter().map(Month::containing).collect())
    }

    async fn find_readings_in_months(
        &self,
        months: &[Month],
    ) -> Result<Vec<ZoneReading>, AppError> {
        if months.is_empty() {
            return Ok(Vec::new());
        }

        zone_readings::Entity::find()
            .filter(zone_readings::Column::Month.is_in(months.iter().map(Month::first_day)))
            .order_by_asc(zone_readings::Column::Month)
            .order_by_asc(zone_readings::Column::ZoneId)
            .all(&self.conn)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(ZoneReading::try_from)
            .collect()
    }

    async fn find_readings_by_zone(&self, zone_id: i32) -> Result<Vec<ZoneReading>, AppError> {
        zone_readings::Entity::find()
            .filter(zone_readings::Column::ZoneId.eq(zone_id))
            .order_by_asc(zone_readings::Column::Month)
            .all(&self.conn)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(ZoneReading::try_from)
            .collect()
    }

    async fn find_sub_zone_readings_in_month(
        &self,
        sub_zone_ids: &[i32],
        month: Month,
    ) -> Result<Vec<SubZoneReading>, AppError> {
        if sub_zone_ids.is_empty() {
            return Ok(Vec::new());
        }

        sub_zone_readings::Entity::find()
            .filter(sub_zone_readings::Column::SubZoneId.is_in(sub_zone_ids.to_vec()))
            .filter(sub_zone_readings::Column::Month.eq(month.first_day()))
            .order_by_asc(sub_zone_readings::Column::SubZoneId)
            .all(&self.conn)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(SubZoneReading::try_from)
            .collect()
    }

    async fn upsert_reading(&self, entity: &ZoneReading) -> Result<ZoneReading, AppError> {
        // The zone and month are the key, so a second push replaces the first.
        let model = zone_readings::Entity::insert(zone_readings::ActiveModel::from(entity))
            .on_conflict(
                OnConflict::columns([zone_readings::Column::ZoneId, zone_readings::Column::Month])
                    .update_columns([
                        zone_readings::Column::Dryness,
                        zone_readings::Column::RainPctOfNormal,
                        zone_readings::Column::GreennessPctVsNormal,
                        zone_readings::Column::WaterNeed,
                        zone_readings::Column::NitrogenHold,
                        zone_readings::Column::BestCrops,
                        zone_readings::Column::Source,
                        zone_readings::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        ZoneReading::try_from(model)
    }

    async fn upsert_sub_zone_reading(
        &self,
        entity: &SubZoneReading,
    ) -> Result<SubZoneReading, AppError> {
        let model = sub_zone_readings::Entity::insert(sub_zone_readings::ActiveModel::from(entity))
            .on_conflict(
                OnConflict::columns([
                    sub_zone_readings::Column::SubZoneId,
                    sub_zone_readings::Column::Month,
                ])
                .update_columns([
                    sub_zone_readings::Column::Dryness,
                    sub_zone_readings::Column::UpdatedAt,
                ])
                .to_owned(),
            )
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        SubZoneReading::try_from(model)
    }
}
