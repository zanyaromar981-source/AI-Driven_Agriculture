use async_trait::async_trait;
use chrono::NaiveDate;
use sea_orm::{
    ActiveValue::NotSet, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, sea_query::OnConflict,
};

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::zones::{
        app::{AppError, ZoneRepository},
        domain::{Month, MonthRange, SubZone, SubZoneReading, Zone, ZoneReading, ZoneSlug},
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

    async fn find_all_sub_zones(&self) -> Result<Vec<SubZone>, AppError> {
        sub_zones::Entity::find()
            .order_by_asc(sub_zones::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(SubZone::try_from)
            .collect()
    }

    async fn find_readings_by_zone_in_range(
        &self,
        zone_id: i32,
        range: MonthRange,
        pagination: &Pagination,
    ) -> Result<(Vec<ZoneReading>, u64), AppError> {
        let query = zone_readings::Entity::find()
            .filter(zone_readings::Column::ZoneId.eq(zone_id))
            .filter(
                zone_readings::Column::Month
                    .between(range.from().first_day(), range.to().first_day()),
            );

        let count = query
            .clone()
            .count(&self.conn)
            .await
            .map_err(database_error)?;

        let readings = query
            .order_by_desc(zone_readings::Column::Month)
            .offset(pagination.skip())
            .limit(*pagination.rows_per_page())
            .all(&self.conn)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(ZoneReading::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok((readings, count))
    }

    async fn insert_reading(&self, entity: &ZoneReading) -> Result<Option<ZoneReading>, AppError> {
        // The unique index on zone and month decides, so of two creates sent
        // at the same moment exactly one gets its row back.
        let inserted = zone_readings::Entity::insert(zone_readings::ActiveModel::from(entity))
            .on_conflict(
                OnConflict::columns([zone_readings::Column::ZoneId, zone_readings::Column::Month])
                    .do_nothing()
                    .to_owned(),
            )
            .exec_with_returning(&self.conn)
            .await;

        match inserted {
            Ok(model) => ZoneReading::try_from(model).map(Some),
            Err(DbErr::RecordNotInserted | DbErr::RecordNotFound(_)) => Ok(None),
            Err(error) => Err(database_error(error)),
        }
    }

    async fn update_reading(&self, entity: &ZoneReading) -> Result<Option<ZoneReading>, AppError> {
        // The key is the filter, never a value to set.
        let mut changes = zone_readings::ActiveModel::from(entity);
        changes.id = NotSet;
        changes.zone_id = NotSet;
        changes.month = NotSet;

        let updated = zone_readings::Entity::update_many()
            .set(changes)
            .filter(zone_readings::Column::ZoneId.eq(*entity.zone_id()))
            .filter(zone_readings::Column::Month.eq(entity.month().first_day()))
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        updated
            .into_iter()
            .next()
            .map(ZoneReading::try_from)
            .transpose()
    }

    async fn delete_reading(&self, zone_id: i32, month: Month) -> Result<bool, AppError> {
        let result = zone_readings::Entity::delete_many()
            .filter(zone_readings::Column::ZoneId.eq(zone_id))
            .filter(zone_readings::Column::Month.eq(month.first_day()))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }

    async fn find_sub_zone_readings_in_range(
        &self,
        sub_zone_id: i32,
        range: MonthRange,
        pagination: &Pagination,
    ) -> Result<(Vec<SubZoneReading>, u64), AppError> {
        let query = sub_zone_readings::Entity::find()
            .filter(sub_zone_readings::Column::SubZoneId.eq(sub_zone_id))
            .filter(
                sub_zone_readings::Column::Month
                    .between(range.from().first_day(), range.to().first_day()),
            );

        let count = query
            .clone()
            .count(&self.conn)
            .await
            .map_err(database_error)?;

        let readings = query
            .order_by_desc(sub_zone_readings::Column::Month)
            .offset(pagination.skip())
            .limit(*pagination.rows_per_page())
            .all(&self.conn)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(SubZoneReading::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok((readings, count))
    }

    async fn insert_sub_zone_reading(
        &self,
        entity: &SubZoneReading,
    ) -> Result<Option<SubZoneReading>, AppError> {
        let inserted =
            sub_zone_readings::Entity::insert(sub_zone_readings::ActiveModel::from(entity))
                .on_conflict(
                    OnConflict::columns([
                        sub_zone_readings::Column::SubZoneId,
                        sub_zone_readings::Column::Month,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .exec_with_returning(&self.conn)
                .await;

        match inserted {
            Ok(model) => SubZoneReading::try_from(model).map(Some),
            Err(DbErr::RecordNotInserted | DbErr::RecordNotFound(_)) => Ok(None),
            Err(error) => Err(database_error(error)),
        }
    }

    async fn update_sub_zone_reading(
        &self,
        entity: &SubZoneReading,
    ) -> Result<Option<SubZoneReading>, AppError> {
        let mut changes = sub_zone_readings::ActiveModel::from(entity);
        changes.id = NotSet;
        changes.sub_zone_id = NotSet;
        changes.month = NotSet;

        let updated = sub_zone_readings::Entity::update_many()
            .set(changes)
            .filter(sub_zone_readings::Column::SubZoneId.eq(*entity.sub_zone_id()))
            .filter(sub_zone_readings::Column::Month.eq(entity.month().first_day()))
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        updated
            .into_iter()
            .next()
            .map(SubZoneReading::try_from)
            .transpose()
    }

    async fn delete_sub_zone_reading(
        &self,
        sub_zone_id: i32,
        month: Month,
    ) -> Result<bool, AppError> {
        let result = sub_zone_readings::Entity::delete_many()
            .filter(sub_zone_readings::Column::SubZoneId.eq(sub_zone_id))
            .filter(sub_zone_readings::Column::Month.eq(month.first_day()))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }
}
