use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect,
    sea_query::{Expr, ExprTrait, OnConflict},
};

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::fires::{
        app::{AppError, FireFilter, FireRepository},
        domain::{Fire, FireCorrection},
        infra::persistence::postgres::entities::fires,
    },
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "fire repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(Debug)]
pub struct FirePostgresRepository {
    conn: DatabaseConnection,
}

impl FirePostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl FireRepository for FirePostgresRepository {
    async fn find_detected_since(&self, since: DateTime<Utc>) -> Result<Vec<Fire>, AppError> {
        let models = fires::Entity::find()
            .filter(fires::Column::DetectedAt.gte(since.naive_utc()))
            .order_by_desc(fires::Column::DetectedAt)
            // Two detections of the same satellite pass share a time; the id
            // keeps their order the same from one request to the next.
            .order_by_desc(fires::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(Fire::try_from).collect()
    }

    async fn upsert(&self, entity: &Fire) -> Result<Fire, AppError> {
        // The external id is the key, so a second push of the same fire
        // replaces the first in one statement and two jobs cannot race.
        let model = fires::Entity::insert(fires::ActiveModel::from(entity))
            .on_conflict(
                OnConflict::column(fires::Column::ExternalId)
                    .update_columns([
                        fires::Column::Lat,
                        fires::Column::Lon,
                        fires::Column::ZoneSlug,
                        fires::Column::PlaceEn,
                        fires::Column::PlaceKu,
                        fires::Column::DetectedAt,
                        fires::Column::AreaHa,
                        fires::Column::WindKmh,
                        fires::Column::WindDirection,
                        fires::Column::Status,
                        fires::Column::FarmsWithin5km,
                        fires::Column::FarmersAlerted,
                        fires::Column::Source,
                        fires::Column::UpdatedAt,
                    ])
                    // A slower copy of a job must not put an older sighting
                    // back over a newer one.
                    .action_and_where(
                        Expr::cust("excluded.detected_at")
                            .gte(Expr::col((fires::Entity, fires::Column::DetectedAt))),
                    )
                    .to_owned(),
            )
            .exec_with_returning(&self.conn)
            .await;

        match model {
            Ok(model) => Fire::try_from(model),
            // Nothing was written because a newer sighting is stored: answer
            // with that one.
            Err(DbErr::RecordNotInserted | DbErr::RecordNotFound(_)) => {
                let kept = fires::Entity::find()
                    .filter(fires::Column::ExternalId.eq(entity.external_id().as_str()))
                    .one(&self.conn)
                    .await
                    .map_err(database_error)?
                    .ok_or(GlobalAppError::NotFound)?;

                Fire::try_from(kept)
            }
            Err(error) => Err(database_error(error)),
        }
    }

    async fn find_page(
        &self,
        filter: &FireFilter,
        pagination: &Pagination,
    ) -> Result<(Vec<Fire>, u64), AppError> {
        let mut query = fires::Entity::find()
            .filter(fires::Column::DetectedAt.gte(filter.span.from().naive_utc()))
            .filter(fires::Column::DetectedAt.lte(filter.span.to().naive_utc()));

        if let Some(status) = filter.status {
            query = query.filter(fires::Column::Status.eq(String::from(status)));
        }

        if let Some(zone_slug) = &filter.zone_slug {
            query = query.filter(fires::Column::ZoneSlug.eq(zone_slug.as_str()));
        }

        let count = query
            .clone()
            .count(&self.conn)
            .await
            .map_err(database_error)?;

        let models = query
            .order_by_desc(fires::Column::DetectedAt)
            .order_by_desc(fires::Column::Id)
            .offset(pagination.skip())
            .limit(*pagination.rows_per_page())
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let fires = models
            .into_iter()
            .map(Fire::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok((fires, count))
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Fire>, AppError> {
        fires::Entity::find_by_id(id)
            .one(&self.conn)
            .await
            .map_err(database_error)?
            .map(Fire::try_from)
            .transpose()
    }

    async fn create(&self, entity: &Fire) -> Result<Option<Fire>, AppError> {
        // The unique index on the external id decides, in this one
        // statement, which of two creates sent at the same moment wins.
        let model = fires::Entity::insert(fires::ActiveModel::from(entity))
            .on_conflict(
                OnConflict::column(fires::Column::ExternalId)
                    .do_nothing()
                    .to_owned(),
            )
            .exec_with_returning(&self.conn)
            .await;

        match model {
            Ok(model) => Ok(Some(Fire::try_from(model)?)),
            Err(DbErr::RecordNotInserted | DbErr::RecordNotFound(_)) => Ok(None),
            Err(error) => Err(database_error(error)),
        }
    }

    async fn update(&self, id: i32, correction: &FireCorrection) -> Result<Option<Fire>, AppError> {
        // No guard on `detected_at` here, unlike the upsert: a person
        // correcting a fire overrides what is stored on purpose.
        let models = fires::Entity::update_many()
            .col_expr(fires::Column::Lat, Expr::value(correction.location().lat()))
            .col_expr(fires::Column::Lon, Expr::value(correction.location().lon()))
            .col_expr(
                fires::Column::ZoneSlug,
                Expr::value(correction.zone_slug().as_ref().map(String::from)),
            )
            .col_expr(
                fires::Column::PlaceEn,
                Expr::value(correction.place_en().as_ref().map(String::from)),
            )
            .col_expr(
                fires::Column::PlaceKu,
                Expr::value(correction.place_ku().as_ref().map(String::from)),
            )
            .col_expr(
                fires::Column::DetectedAt,
                Expr::value(correction.detected_at().naive_utc()),
            )
            .col_expr(fires::Column::AreaHa, Expr::value(*correction.area_ha()))
            .col_expr(fires::Column::WindKmh, Expr::value(*correction.wind_kmh()))
            .col_expr(
                fires::Column::WindDirection,
                Expr::value(correction.wind_direction().map(String::from)),
            )
            .col_expr(
                fires::Column::Status,
                Expr::value(String::from(*correction.status())),
            )
            .col_expr(
                fires::Column::FarmsWithin5km,
                Expr::value(*correction.farms_within_5km()),
            )
            .col_expr(
                fires::Column::FarmersAlerted,
                Expr::value(*correction.farmers_alerted()),
            )
            .col_expr(
                fires::Column::Source,
                Expr::value(String::from(correction.source())),
            )
            .col_expr(
                fires::Column::UpdatedAt,
                Expr::value(correction.updated_at().naive_utc()),
            )
            .filter(fires::Column::Id.eq(id))
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().next().map(Fire::try_from).transpose()
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        fires::Entity::delete_by_id(id)
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(())
    }
}
