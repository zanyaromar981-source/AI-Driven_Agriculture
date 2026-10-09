use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    sea_query::{Expr, OnConflict},
};

use crate::{
    app::AppError as GlobalAppError,
    features::alerts::{
        app::{AlertRepository, AppError, DeviceRepository},
        domain::{Alert, AlertLevel, Device, PushToken},
        infra::persistence::postgres::{
            entities::{alerts, devices},
            mappings::token_hash,
        },
    },
    shared::Phone,
};

/// Far below the 65,535 bind parameters Postgres takes per statement.
const ID_CHUNK: usize = 10_000;

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "alert repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(Debug)]
pub struct AlertPostgresRepository {
    conn: DatabaseConnection,
}

impl AlertPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl AlertRepository for AlertPostgresRepository {
    async fn upsert(&self, entity: &Alert) -> Result<Alert, AppError> {
        // `pushed`, `done`, their times and `created_at` are left out of the
        // update on purpose: they belong to the farmer and the push sender,
        // not to the job that sends the alert again.
        let model = alerts::Entity::insert(alerts::ActiveModel::from(entity))
            .on_conflict(
                OnConflict::columns([alerts::Column::FarmId, alerts::Column::Key])
                    .update_columns([
                        alerts::Column::Type,
                        alerts::Column::Day,
                        alerts::Column::Level,
                        alerts::Column::Confidence,
                        alerts::Column::Ku,
                        alerts::Column::En,
                        alerts::Column::ActionKu,
                        alerts::Column::ActionEn,
                        alerts::Column::Source,
                        alerts::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        Alert::try_from(model)
    }

    async fn find_by_farms(
        &self,
        farm_ids: &[i32],
        first_day: NaiveDate,
    ) -> Result<Vec<Alert>, AppError> {
        let mut found = Vec::new();

        for chunk in farm_ids.chunks(ID_CHUNK) {
            let models = alerts::Entity::find()
                .filter(alerts::Column::FarmId.is_in(chunk.to_vec()))
                .filter(alerts::Column::Day.gte(first_day))
                .all(&self.conn)
                .await
                .map_err(database_error)?;

            found.extend(models);
        }

        // Sorted here, not in SQL, so the order also holds across chunks.
        found.sort_by(|a, b| b.day.cmp(&a.day).then(b.id.cmp(&a.id)));

        found.into_iter().map(Alert::try_from).collect()
    }

    async fn find_all_by_farm(&self, farm_id: i32) -> Result<Vec<Alert>, AppError> {
        let models = alerts::Entity::find()
            .filter(alerts::Column::FarmId.eq(farm_id))
            .order_by_desc(alerts::Column::Day)
            .order_by_desc(alerts::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(Alert::try_from).collect()
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Alert>, AppError> {
        alerts::Entity::find_by_id(id)
            .one(&self.conn)
            .await
            .map_err(database_error)?
            .map(Alert::try_from)
            .transpose()
    }

    async fn mark_done(&self, id: i32, now: DateTime<Utc>) -> Result<bool, AppError> {
        // COALESCE keeps the first time, so a repeat changes nothing and
        // still counts as found.
        let result = alerts::Entity::update_many()
            .col_expr(alerts::Column::Done, Expr::value(true))
            .col_expr(
                alerts::Column::DoneAt,
                Expr::cust_with_values("COALESCE(done_at, $1)", [now.naive_utc()]),
            )
            .filter(alerts::Column::Id.eq(id))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }

    async fn mark_pushed(&self, id: i32, now: DateTime<Utc>) -> Result<bool, AppError> {
        let result = alerts::Entity::update_many()
            .col_expr(alerts::Column::Pushed, Expr::value(true))
            .col_expr(
                alerts::Column::PushedAt,
                Expr::cust_with_values("COALESCE(pushed_at, $1)", [now.naive_utc()]),
            )
            .filter(alerts::Column::Id.eq(id))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }

    async fn find_unpushed_alarms(&self, first_day: NaiveDate) -> Result<Vec<Alert>, AppError> {
        let models = alerts::Entity::find()
            .filter(alerts::Column::Level.eq(String::from(AlertLevel::Alarm)))
            .filter(alerts::Column::Pushed.eq(false))
            .filter(alerts::Column::Done.eq(false))
            .filter(alerts::Column::Day.gte(first_day))
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(Alert::try_from).collect()
    }

    async fn find_farms_pushed_since(&self, since: DateTime<Utc>) -> Result<Vec<i32>, AppError> {
        alerts::Entity::find()
            .select_only()
            .column(alerts::Column::FarmId)
            .distinct()
            .filter(alerts::Column::PushedAt.gte(since.naive_utc()))
            .into_tuple()
            .all(&self.conn)
            .await
            .map_err(database_error)
    }

    async fn delete(&self, id: i32) -> Result<bool, AppError> {
        let result = alerts::Entity::delete_many()
            .filter(alerts::Column::Id.eq(id))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }

    async fn delete_by_farms(&self, farm_ids: &[i32]) -> Result<u64, AppError> {
        let mut removed = 0;

        for chunk in farm_ids.chunks(ID_CHUNK) {
            removed += alerts::Entity::delete_many()
                .filter(alerts::Column::FarmId.is_in(chunk.to_vec()))
                .exec(&self.conn)
                .await
                .map_err(database_error)?
                .rows_affected;
        }

        Ok(removed)
    }
}

#[derive(Debug)]
pub struct DevicePostgresRepository {
    conn: DatabaseConnection,
}

impl DevicePostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl DeviceRepository for DevicePostgresRepository {
    async fn upsert(&self, entity: &Device) -> Result<(), AppError> {
        // One statement: the unique index on the token's hash decides, also
        // between two phones' registrations sent at the same moment, and the
        // token ends up under exactly one number.
        devices::Entity::insert(devices::ActiveModel::from(entity))
            .on_conflict(
                OnConflict::column(devices::Column::TokenHash)
                    .update_columns([
                        devices::Column::Phone,
                        devices::Column::Platform,
                        devices::Column::Lang,
                        devices::Column::RedAlerts,
                        devices::Column::WeeklyPlan,
                        devices::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec_without_returning(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn delete(&self, push_token: &PushToken, phone: &Phone) -> Result<(), AppError> {
        devices::Entity::delete_many()
            .filter(devices::Column::TokenHash.eq(token_hash(push_token)))
            .filter(devices::Column::Phone.eq(phone.as_str()))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn delete_by_token(&self, push_token: &PushToken) -> Result<(), AppError> {
        devices::Entity::delete_many()
            .filter(devices::Column::TokenHash.eq(token_hash(push_token)))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn find_alert_devices(&self, phone: &Phone) -> Result<Vec<Device>, AppError> {
        let models = devices::Entity::find()
            .filter(devices::Column::Phone.eq(phone.as_str()))
            .filter(devices::Column::RedAlerts.eq(true))
            .order_by_asc(devices::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(Device::try_from).collect()
    }

    async fn delete_all_by_phone(&self, phone: &Phone) -> Result<u64, AppError> {
        let result = devices::Entity::delete_many()
            .filter(devices::Column::Phone.eq(phone.as_str()))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected)
    }
}
