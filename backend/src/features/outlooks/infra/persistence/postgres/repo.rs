use async_trait::async_trait;
use chrono::NaiveDate;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    sea_query::OnConflict,
};

use crate::{
    app::AppError as GlobalAppError,
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
}
