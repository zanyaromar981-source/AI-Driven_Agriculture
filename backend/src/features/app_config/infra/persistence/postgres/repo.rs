use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, DatabaseConnection, DbBackend, DbErr, EntityTrait, ExprTrait, FromQueryResult,
    QueryFilter, Statement,
    sea_query::{Expr, OnConflict},
};

use crate::{
    app::AppError as GlobalAppError,
    features::app_config::{
        app::{AppConfigRepository, AppError},
        domain::{AppConfig, AppVersion},
        infra::persistence::postgres::{
            entities::{app_config, app_versions_seen},
            mappings::THE_ROW,
        },
    },
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "app config repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

fn missing_row() -> AppError {
    tracing::error!("the app_config row is missing: the migration seeds it");

    GlobalAppError::MissingValue("The app config row is missing".to_string()).into()
}

#[derive(FromQueryResult)]
struct VersionCount {
    version: String,
    farmers: i64,
}

#[derive(Debug)]
pub struct AppConfigPostgresRepository {
    conn: DatabaseConnection,
}

impl AppConfigPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl AppConfigRepository for AppConfigPostgresRepository {
    async fn find(&self) -> Result<AppConfig, AppError> {
        app_config::Entity::find_by_id(THE_ROW)
            .one(&self.conn)
            .await
            .map_err(database_error)?
            .ok_or_else(missing_row)?
            .try_into()
    }

    async fn replace(&self, config: &AppConfig) -> Result<AppConfig, AppError> {
        // One statement replaces every column of the one row. Two saves at
        // the same moment leave one of the two whole, never a mix.
        let updated = app_config::Entity::update_many()
            .set(app_config::ActiveModel::from(config))
            .filter(app_config::Column::Id.eq(THE_ROW))
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        updated
            .into_iter()
            .next()
            .ok_or_else(missing_row)?
            .try_into()
    }

    async fn record_sighting(
        &self,
        farmer_id: i32,
        version: &AppVersion,
        now: DateTime<Utc>,
        unless_after: DateTime<Utc>,
    ) -> Result<(), AppError> {
        // The farmer and the version are the key. A row seen recently is
        // left alone by the statement itself, so however many servers ask,
        // it is rewritten once an hour at most.
        app_versions_seen::Entity::insert(app_versions_seen::ActiveModel {
            farmer_id: Set(farmer_id),
            version: Set(version.to_string()),
            last_seen: Set(now.naive_utc()),
        })
        .on_conflict(
            OnConflict::columns([
                app_versions_seen::Column::FarmerId,
                app_versions_seen::Column::Version,
            ])
            .update_column(app_versions_seen::Column::LastSeen)
            .action_and_where(
                Expr::col((
                    app_versions_seen::Entity,
                    app_versions_seen::Column::LastSeen,
                ))
                .lte(unless_after.naive_utc()),
            )
            .to_owned(),
        )
        .exec_without_returning(&self.conn)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn count_farmers_by_version(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<(AppVersion, u64)>, AppError> {
        // Each farmer once, under the version they were seen on last.
        let rows = VersionCount::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT latest.version AS version, COUNT(*) AS farmers
               FROM (
                 SELECT DISTINCT ON (farmer_id) farmer_id, version
                   FROM app_versions_seen
                  WHERE last_seen >= $1
                  ORDER BY farmer_id, last_seen DESC, version DESC
               ) AS latest
              GROUP BY latest.version",
            [since.naive_utc().into()],
        ))
        .all(&self.conn)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(|row| {
                Ok((
                    AppVersion::new(&row.version)?,
                    u64::try_from(row.farmers).unwrap_or_default(),
                ))
            })
            .collect()
    }
}
