use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect,
    sea_query::{Expr, OnConflict},
};

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::briefs::{
        app::{AppError, BriefRepository},
        domain::{BriefScope, DailyBrief, DayRange, FarmZone, ZoneSlug},
        infra::persistence::postgres::{
            entities::{daily_briefs, farm_brief_zones},
            mappings::{stored_points, stored_sources},
        },
    },
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "brief repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(Debug)]
pub struct BriefPostgresRepository {
    conn: DatabaseConnection,
}

impl BriefPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl BriefRepository for BriefPostgresRepository {
    async fn find_latest(&self, scope: &BriefScope) -> Result<Option<DailyBrief>, AppError> {
        daily_briefs::Entity::find()
            .filter(daily_briefs::Column::Scope.eq(scope.as_str()))
            .order_by_desc(daily_briefs::Column::Day)
            .one(&self.conn)
            .await
            .map_err(database_error)?
            .map(DailyBrief::try_from)
            .transpose()
    }

    async fn find_between(
        &self,
        scope: &BriefScope,
        range: &DayRange,
    ) -> Result<Vec<DailyBrief>, AppError> {
        let models = daily_briefs::Entity::find()
            .filter(daily_briefs::Column::Scope.eq(scope.as_str()))
            .filter(daily_briefs::Column::Day.gte(*range.from()))
            .filter(daily_briefs::Column::Day.lte(*range.to()))
            .order_by_desc(daily_briefs::Column::Day)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(DailyBrief::try_from).collect()
    }

    async fn upsert(&self, entity: &DailyBrief) -> Result<DailyBrief, AppError> {
        // Day and scope together are the key, so a repeat of the push, or
        // two copies of the job at the same moment, replace the brief in
        // one statement. The day is in the key, so there is no older push
        // to guard against: the last one for a day is the brief of that day.
        let model = daily_briefs::Entity::insert(daily_briefs::ActiveModel::from(entity))
            .on_conflict(
                OnConflict::columns([daily_briefs::Column::Day, daily_briefs::Column::Scope])
                    .update_columns([
                        daily_briefs::Column::HeadlineEn,
                        daily_briefs::Column::HeadlineKu,
                        daily_briefs::Column::SummaryEn,
                        daily_briefs::Column::SummaryKu,
                        daily_briefs::Column::Points,
                        daily_briefs::Column::Sources,
                        daily_briefs::Column::Author,
                        daily_briefs::Column::GeneratedAt,
                        daily_briefs::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        DailyBrief::try_from(model)
    }

    async fn delete(&self, day: NaiveDate, scope: &BriefScope) -> Result<(), AppError> {
        daily_briefs::Entity::delete_many()
            .filter(daily_briefs::Column::Day.eq(day))
            .filter(daily_briefs::Column::Scope.eq(scope.as_str()))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn find_farm_zone(&self, farm_id: i32) -> Result<Option<ZoneSlug>, AppError> {
        farm_brief_zones::Entity::find()
            .filter(farm_brief_zones::Column::FarmId.eq(farm_id))
            .one(&self.conn)
            .await
            .map_err(database_error)?
            .map(|model| Ok(ZoneSlug::new(model.zone_slug)?))
            .transpose()
    }

    async fn upsert_farm_zones(&self, zones: &[FarmZone]) -> Result<u64, AppError> {
        // One statement for the whole push, so it lands as a whole or not
        // at all. A push holds at most 2,000 farms of three values each,
        // far below the 65,535 parameters a statement takes.
        let recorded_at = Utc::now().naive_utc();

        farm_brief_zones::Entity::insert_many(
            zones
                .iter()
                .map(|zone| farm_brief_zones::ActiveModel::from((zone, recorded_at))),
        )
        .on_conflict(
            OnConflict::column(farm_brief_zones::Column::FarmId)
                .update_columns([
                    farm_brief_zones::Column::ZoneSlug,
                    farm_brief_zones::Column::UpdatedAt,
                ])
                .to_owned(),
        )
        .exec_without_returning(&self.conn)
        .await
        .map_err(database_error)?;

        Ok(zones.len() as u64)
    }

    async fn find_page(
        &self,
        scope: Option<&BriefScope>,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        pagination: &Pagination,
    ) -> Result<(Vec<DailyBrief>, u64), AppError> {
        let mut query = daily_briefs::Entity::find();

        if let Some(scope) = scope {
            query = query.filter(daily_briefs::Column::Scope.eq(scope.as_str()));
        }

        if let Some(from) = from {
            query = query.filter(daily_briefs::Column::Day.gte(from));
        }

        if let Some(to) = to {
            query = query.filter(daily_briefs::Column::Day.lte(to));
        }

        let count = query
            .clone()
            .count(&self.conn)
            .await
            .map_err(database_error)?;

        // The scope breaks the tie between the briefs of one day, so the
        // pages do not shuffle between two requests.
        let models = query
            .order_by_desc(daily_briefs::Column::Day)
            .order_by_asc(daily_briefs::Column::Scope)
            .offset(pagination.skip())
            .limit(*pagination.rows_per_page())
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        Ok((
            models
                .into_iter()
                .map(DailyBrief::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            count,
        ))
    }

    async fn update(&self, entity: &DailyBrief) -> Result<Option<DailyBrief>, AppError> {
        // The update decides by itself whether there is a brief to replace:
        // no row comes back when the day and scope have none.
        let models = daily_briefs::Entity::update_many()
            .col_expr(
                daily_briefs::Column::HeadlineEn,
                Expr::value(String::from(entity.headline_en())),
            )
            .col_expr(
                daily_briefs::Column::HeadlineKu,
                Expr::value(String::from(entity.headline_ku())),
            )
            .col_expr(
                daily_briefs::Column::SummaryEn,
                Expr::value(String::from(entity.summary_en())),
            )
            .col_expr(
                daily_briefs::Column::SummaryKu,
                Expr::value(String::from(entity.summary_ku())),
            )
            .col_expr(
                daily_briefs::Column::Points,
                Expr::value(stored_points(entity.points())),
            )
            .col_expr(
                daily_briefs::Column::Sources,
                Expr::value(stored_sources(entity.sources())),
            )
            .col_expr(
                daily_briefs::Column::Author,
                Expr::value(String::from(entity.author())),
            )
            .col_expr(
                daily_briefs::Column::GeneratedAt,
                Expr::value(entity.generated_at().naive_utc()),
            )
            .col_expr(
                daily_briefs::Column::UpdatedAt,
                Expr::value(entity.updated_at().naive_utc()),
            )
            .filter(daily_briefs::Column::Day.eq(*entity.day()))
            .filter(daily_briefs::Column::Scope.eq(entity.scope().as_str()))
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        models
            .into_iter()
            .next()
            .map(DailyBrief::try_from)
            .transpose()
    }
}
