use async_trait::async_trait;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    TryInsertResult,
    sea_query::{Expr, OnConflict},
};

use crate::{
    app::AppError as GlobalAppError,
    features::water::{
        app::{AppError, WaterPlanRepository},
        domain::{Season, WaterPlanEntry, ZoneSlug},
        infra::persistence::postgres::entities::water_plan_entries,
    },
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "water plan repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(Debug)]
pub struct WaterPlanPostgresRepository {
    conn: DatabaseConnection,
}

impl WaterPlanPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl WaterPlanRepository for WaterPlanPostgresRepository {
    async fn find_latest_season(&self) -> Result<Option<Season>, AppError> {
        // The season text sorts in time order, see `Season`.
        let model = water_plan_entries::Entity::find()
            .order_by_desc(water_plan_entries::Column::Season)
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(model.map(|model| Season::new(model.season)).transpose()?)
    }

    async fn find_by_season(&self, season: &Season) -> Result<Vec<WaterPlanEntry>, AppError> {
        let models = water_plan_entries::Entity::find()
            .filter(water_plan_entries::Column::Season.eq(season.as_str()))
            .order_by_asc(water_plan_entries::Column::ZoneSlug)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(WaterPlanEntry::try_from).collect()
    }

    async fn upsert(&self, entry: &WaterPlanEntry) -> Result<WaterPlanEntry, AppError> {
        // The season and the zone are the key, so sending a zone again
        // replaces its line instead of giving it a second one.
        let model =
            water_plan_entries::Entity::insert(water_plan_entries::ActiveModel::from(entry))
                .on_conflict(
                    OnConflict::columns([
                        water_plan_entries::Column::Season,
                        water_plan_entries::Column::ZoneSlug,
                    ])
                    .update_columns([
                        water_plan_entries::Column::Need,
                        water_plan_entries::Column::DamSlug,
                        water_plan_entries::Column::SendMillionM3,
                        water_plan_entries::Column::Urgent,
                        water_plan_entries::Column::NoteEn,
                        water_plan_entries::Column::NoteKu,
                        water_plan_entries::Column::UpdatedAt,
                    ])
                    .to_owned(),
                )
                .exec_with_returning(&self.conn)
                .await
                .map_err(database_error)?;

        WaterPlanEntry::try_from(model)
    }

    async fn delete(&self, season: &Season, zone_slug: &ZoneSlug) -> Result<bool, AppError> {
        let result = water_plan_entries::Entity::delete_many()
            .filter(water_plan_entries::Column::Season.eq(season.as_str()))
            .filter(water_plan_entries::Column::ZoneSlug.eq(zone_slug.as_str()))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }

    async fn find_seasons(&self) -> Result<Vec<Season>, AppError> {
        // The season text sorts in time order, see `Season`.
        let seasons: Vec<String> = water_plan_entries::Entity::find()
            .select_only()
            .column(water_plan_entries::Column::Season)
            .distinct()
            .order_by_desc(water_plan_entries::Column::Season)
            .into_tuple()
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(seasons
            .into_iter()
            .map(Season::new)
            .collect::<Result<Vec<_>, _>>()?)
    }

    async fn create(&self, entry: &WaterPlanEntry) -> Result<Option<WaterPlanEntry>, AppError> {
        // One statement decides: the unique index on the season and the zone
        // lets exactly one of two racing creates insert, and the other gets
        // no row back.
        let inserted =
            water_plan_entries::Entity::insert(water_plan_entries::ActiveModel::from(entry))
                .on_conflict(
                    OnConflict::columns([
                        water_plan_entries::Column::Season,
                        water_plan_entries::Column::ZoneSlug,
                    ])
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
                models.pop().map(WaterPlanEntry::try_from).transpose()
            }
            TryInsertResult::Conflicted | TryInsertResult::Empty => Ok(None),
        }
    }

    async fn update(&self, entry: &WaterPlanEntry) -> Result<Option<WaterPlanEntry>, AppError> {
        let mut models = water_plan_entries::Entity::update_many()
            .col_expr(
                water_plan_entries::Column::Need,
                Expr::value(entry.need().value()),
            )
            .col_expr(
                water_plan_entries::Column::DamSlug,
                Expr::value(entry.dam_slug().as_ref().map(String::from)),
            )
            .col_expr(
                water_plan_entries::Column::SendMillionM3,
                Expr::value(*entry.send_million_m3()),
            )
            .col_expr(
                water_plan_entries::Column::Urgent,
                Expr::value(*entry.urgent()),
            )
            .col_expr(
                water_plan_entries::Column::NoteEn,
                Expr::value(entry.note_en().as_ref().map(String::from)),
            )
            .col_expr(
                water_plan_entries::Column::NoteKu,
                Expr::value(entry.note_ku().as_ref().map(String::from)),
            )
            .col_expr(
                water_plan_entries::Column::UpdatedAt,
                Expr::value(entry.updated_at().naive_utc()),
            )
            .filter(water_plan_entries::Column::Season.eq(entry.season().as_str()))
            .filter(water_plan_entries::Column::ZoneSlug.eq(entry.zone_slug().as_str()))
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        // The key is unique, so the update touched one row or none.
        models.pop().map(WaterPlanEntry::try_from).transpose()
    }
}
