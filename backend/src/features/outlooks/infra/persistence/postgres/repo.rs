use async_trait::async_trait;
use chrono::NaiveDate;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, TryInsertResult,
    sea_query::{Expr, OnConflict},
};

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::outlooks::{
        app::{AppError, OutlookRepository},
        domain::{IssueMonth, OutlookRun, Season, ZoneOutlook, ZoneSlug},
        infra::persistence::postgres::entities::{outlook_runs, season_outlooks},
    },
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "outlook repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(Debug)]
pub struct OutlookPostgresRepository {
    conn: DatabaseConnection,
}

impl OutlookPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl OutlookRepository for OutlookPostgresRepository {
    async fn find_latest_season(&self) -> Result<Option<Season>, AppError> {
        // The season text sorts in time order, see `Season`.
        let model = season_outlooks::Entity::find()
            .order_by_desc(season_outlooks::Column::Season)
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(model.map(|model| Season::new(model.season)).transpose()?)
    }

    async fn find_issue_months(&self, season: &Season) -> Result<Vec<IssueMonth>, AppError> {
        let months: Vec<NaiveDate> = season_outlooks::Entity::find()
            .select_only()
            .column(season_outlooks::Column::Issued)
            .distinct()
            .filter(season_outlooks::Column::Season.eq(season.as_str()))
            .order_by_asc(season_outlooks::Column::Issued)
            .into_tuple()
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(months
            .into_iter()
            .map(IssueMonth::from_date)
            .collect::<Result<Vec<_>, _>>()?)
    }

    async fn find_zone_outlooks(
        &self,
        season: &Season,
        issued: &IssueMonth,
    ) -> Result<Vec<ZoneOutlook>, AppError> {
        let models = season_outlooks::Entity::find()
            .filter(season_outlooks::Column::Season.eq(season.as_str()))
            .filter(season_outlooks::Column::Issued.eq(issued.first_day()))
            .order_by_asc(season_outlooks::Column::ZoneSlug)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(ZoneOutlook::try_from).collect()
    }

    async fn find_zone_history(
        &self,
        zone_slug: &ZoneSlug,
        season: &Season,
    ) -> Result<Vec<ZoneOutlook>, AppError> {
        let models = season_outlooks::Entity::find()
            .filter(season_outlooks::Column::ZoneSlug.eq(zone_slug.as_str()))
            .filter(season_outlooks::Column::Season.eq(season.as_str()))
            .order_by_asc(season_outlooks::Column::Issued)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(ZoneOutlook::try_from).collect()
    }

    async fn find_run(
        &self,
        season: &Season,
        issued: &IssueMonth,
    ) -> Result<Option<OutlookRun>, AppError> {
        let model = outlook_runs::Entity::find()
            .filter(outlook_runs::Column::Season.eq(season.as_str()))
            .filter(outlook_runs::Column::Issued.eq(issued.first_day()))
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        model.map(OutlookRun::try_from).transpose()
    }

    async fn upsert_zone_outlook(&self, outlook: &ZoneOutlook) -> Result<ZoneOutlook, AppError> {
        // The zone, the season and the month are the key, so a job that runs
        // twice for the same issue replaces its earlier outlook.
        let model = season_outlooks::Entity::insert(season_outlooks::ActiveModel::from(outlook))
            .on_conflict(
                OnConflict::columns([
                    season_outlooks::Column::ZoneSlug,
                    season_outlooks::Column::Season,
                    season_outlooks::Column::Issued,
                ])
                .update_columns([
                    season_outlooks::Column::Outlook,
                    season_outlooks::Column::ConfidencePct,
                    season_outlooks::Column::ReasonEn,
                    season_outlooks::Column::ReasonKu,
                    season_outlooks::Column::UpdatedAt,
                ])
                .to_owned(),
            )
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        ZoneOutlook::try_from(model)
    }

    async fn upsert_run(&self, run: &OutlookRun) -> Result<OutlookRun, AppError> {
        let model = outlook_runs::Entity::insert(outlook_runs::ActiveModel::from(run))
            .on_conflict(
                OnConflict::columns([outlook_runs::Column::Season, outlook_runs::Column::Issued])
                    .update_columns([
                        outlook_runs::Column::SeasonsTested,
                        outlook_runs::Column::SeasonsRight,
                        outlook_runs::Column::Method,
                        outlook_runs::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        OutlookRun::try_from(model)
    }

    async fn find_zone_outlooks_page(
        &self,
        season: Option<&Season>,
        issued: Option<&IssueMonth>,
        pagination: &Pagination,
    ) -> Result<(Vec<ZoneOutlook>, u64), AppError> {
        let mut query = season_outlooks::Entity::find();

        if let Some(season) = season {
            query = query.filter(season_outlooks::Column::Season.eq(season.as_str()));
        }

        if let Some(issued) = issued {
            query = query.filter(season_outlooks::Column::Issued.eq(issued.first_day()));
        }

        let count = query
            .clone()
            .count(&self.conn)
            .await
            .map_err(database_error)?;

        // The three columns are the unique key, so the order is total and a
        // row cannot fall between two pages.
        let models = query
            .order_by_desc(season_outlooks::Column::Issued)
            .order_by_desc(season_outlooks::Column::Season)
            .order_by_asc(season_outlooks::Column::ZoneSlug)
            .offset(pagination.skip())
            .limit(*pagination.rows_per_page())
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        Ok((
            models
                .into_iter()
                .map(ZoneOutlook::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            count,
        ))
    }

    async fn create_zone_outlook(
        &self,
        outlook: &ZoneOutlook,
    ) -> Result<Option<ZoneOutlook>, AppError> {
        // One statement decides: the unique index lets exactly one of two
        // racing creates insert, and the other gets no row back.
        let inserted = season_outlooks::Entity::insert(season_outlooks::ActiveModel::from(outlook))
            .on_conflict(
                OnConflict::columns([
                    season_outlooks::Column::ZoneSlug,
                    season_outlooks::Column::Season,
                    season_outlooks::Column::Issued,
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
                models.pop().map(ZoneOutlook::try_from).transpose()
            }
            TryInsertResult::Conflicted | TryInsertResult::Empty => Ok(None),
        }
    }

    async fn update_zone_outlook(
        &self,
        outlook: &ZoneOutlook,
    ) -> Result<Option<ZoneOutlook>, AppError> {
        let mut models = season_outlooks::Entity::update_many()
            .col_expr(
                season_outlooks::Column::Outlook,
                Expr::value(String::from(*outlook.outlook())),
            )
            .col_expr(
                season_outlooks::Column::ConfidencePct,
                Expr::value(outlook.confidence().value()),
            )
            .col_expr(
                season_outlooks::Column::ReasonEn,
                Expr::value(outlook.reason_en().as_ref().map(String::from)),
            )
            .col_expr(
                season_outlooks::Column::ReasonKu,
                Expr::value(outlook.reason_ku().as_ref().map(String::from)),
            )
            .col_expr(
                season_outlooks::Column::UpdatedAt,
                Expr::value(outlook.updated_at().naive_utc()),
            )
            .filter(season_outlooks::Column::ZoneSlug.eq(outlook.zone_slug().as_str()))
            .filter(season_outlooks::Column::Season.eq(outlook.season().as_str()))
            .filter(season_outlooks::Column::Issued.eq(outlook.issued().first_day()))
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        // The key is unique, so the update touched one row or none.
        models.pop().map(ZoneOutlook::try_from).transpose()
    }

    async fn delete_zone_outlook(
        &self,
        zone_slug: &ZoneSlug,
        season: &Season,
        issued: &IssueMonth,
    ) -> Result<bool, AppError> {
        let result = season_outlooks::Entity::delete_many()
            .filter(season_outlooks::Column::ZoneSlug.eq(zone_slug.as_str()))
            .filter(season_outlooks::Column::Season.eq(season.as_str()))
            .filter(season_outlooks::Column::Issued.eq(issued.first_day()))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }

    async fn find_runs(&self) -> Result<Vec<OutlookRun>, AppError> {
        let models = outlook_runs::Entity::find()
            .order_by_desc(outlook_runs::Column::Issued)
            .order_by_desc(outlook_runs::Column::Season)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(OutlookRun::try_from).collect()
    }

    async fn create_run(&self, run: &OutlookRun) -> Result<Option<OutlookRun>, AppError> {
        let inserted = outlook_runs::Entity::insert(outlook_runs::ActiveModel::from(run))
            .on_conflict(
                OnConflict::columns([outlook_runs::Column::Season, outlook_runs::Column::Issued])
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
                models.pop().map(OutlookRun::try_from).transpose()
            }
            TryInsertResult::Conflicted | TryInsertResult::Empty => Ok(None),
        }
    }

    async fn update_run(&self, run: &OutlookRun) -> Result<Option<OutlookRun>, AppError> {
        let mut models = outlook_runs::Entity::update_many()
            .col_expr(
                outlook_runs::Column::SeasonsTested,
                Expr::value(*run.seasons_tested()),
            )
            .col_expr(
                outlook_runs::Column::SeasonsRight,
                Expr::value(*run.seasons_right()),
            )
            .col_expr(
                outlook_runs::Column::Method,
                Expr::value(run.method().as_str()),
            )
            .col_expr(
                outlook_runs::Column::UpdatedAt,
                Expr::value(run.updated_at().naive_utc()),
            )
            .filter(outlook_runs::Column::Season.eq(run.season().as_str()))
            .filter(outlook_runs::Column::Issued.eq(run.issued().first_day()))
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        models.pop().map(OutlookRun::try_from).transpose()
    }

    async fn delete_run(&self, season: &Season, issued: &IssueMonth) -> Result<bool, AppError> {
        let result = outlook_runs::Entity::delete_many()
            .filter(outlook_runs::Column::Season.eq(season.as_str()))
            .filter(outlook_runs::Column::Issued.eq(issued.first_day()))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }
}
