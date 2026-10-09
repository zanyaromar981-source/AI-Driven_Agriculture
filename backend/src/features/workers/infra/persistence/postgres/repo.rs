use async_trait::async_trait;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, Order, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Value,
    sea_query::{
        Condition, Expr, NullOrdering, OnConflict, SimpleExpr, extension::postgres::PgExpr,
    },
};

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::workers::{
        app::{AppError, WorkerFilter, WorkerRepository},
        domain::{GeoPoint, Worker},
        infra::persistence::postgres::entities::workers,
    },
    shared::Phone,
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "worker repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

/// The kilometres from `from` to a card's point: the same haversine formula
/// and radius the Alwa listing list and `GeoPoint::km_to` use, so the order
/// of the page and the distance written on each card agree. Null for a card
/// without a point.
fn km_from(from: &GeoPoint) -> SimpleExpr {
    Expr::cust_with_values(
        "2 * 6371.0 * asin(least(1.0, sqrt(\
            power(sin(radians(workers.lat - $1) / 2), 2) \
            + cos(radians($2)) * cos(radians(workers.lat)) \
            * power(sin(radians(workers.lon - $3) / 2), 2))))",
        [from.lat(), from.lat(), from.lon()],
    )
}

/// The phones travel as one array, a single bind parameter however many
/// accounts are blocked.
fn not_one_of(phones: &[Phone]) -> SimpleExpr {
    let phones: Vec<String> = phones.iter().map(String::from).collect();

    Expr::cust_with_values("workers.phone <> ALL($1)", [Value::from(phones)])
}

#[derive(Debug)]
pub struct WorkerPostgresRepository {
    conn: DatabaseConnection,
}

impl WorkerPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl WorkerRepository for WorkerPostgresRepository {
    async fn save(&self, entity: &Worker) -> Result<Worker, AppError> {
        // The unique phone decides between creating and replacing, inside
        // the one statement. `created_at` is the only column a replace
        // leaves alone.
        let model = workers::Entity::insert(workers::ActiveModel::from(entity))
            .on_conflict(
                OnConflict::column(workers::Column::Phone)
                    .update_columns([
                        workers::Column::Name,
                        workers::Column::CostIqd,
                        workers::Column::CostPer,
                        workers::Column::Note,
                        workers::Column::ZoneSlug,
                        workers::Column::Lat,
                        workers::Column::Lon,
                        workers::Column::Available,
                        workers::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        Worker::try_from(model)
    }

    async fn find_by_phone(&self, phone: &Phone) -> Result<Option<Worker>, AppError> {
        workers::Entity::find()
            .filter(workers::Column::Phone.eq(phone.as_str()))
            .one(&self.conn)
            .await
            .map_err(database_error)?
            .map(Worker::try_from)
            .transpose()
    }

    async fn delete_by_phone(&self, phone: &Phone) -> Result<bool, AppError> {
        let result = workers::Entity::delete_many()
            .filter(workers::Column::Phone.eq(phone.as_str()))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }

    async fn find_page(
        &self,
        filter: &WorkerFilter,
        near: Option<&GeoPoint>,
        pagination: &Pagination,
    ) -> Result<(Vec<Worker>, u64), AppError> {
        let mut query = workers::Entity::find();

        if let Some(zone) = &filter.zone {
            query = query.filter(workers::Column::ZoneSlug.eq(zone.as_str()));
        }

        if let Some(search) = &filter.search {
            // The pattern escapes `%` and `_` with a backslash, which is
            // the escape character Postgres uses when none is named.
            let pattern = search.like_pattern();

            query = query.filter(
                Condition::any()
                    .add(Expr::col((workers::Entity, workers::Column::Name)).ilike(pattern.clone()))
                    .add(Expr::col((workers::Entity, workers::Column::Note)).ilike(pattern)),
            );
        }

        if let Some(max_cost) = filter.max_cost {
            query = query.filter(workers::Column::CostIqd.lte(max_cost.value()));
        }

        if let Some(cost_per) = filter.cost_per {
            query = query.filter(workers::Column::CostPer.eq(String::from(cost_per)));
        }

        if let Some(available) = filter.available {
            query = query.filter(workers::Column::Available.eq(available));
        }

        if !filter.excluding.is_empty() {
            query = query.filter(not_one_of(&filter.excluding));
        }

        let count = query
            .clone()
            .count(&self.conn)
            .await
            .map_err(database_error)?;

        // The distance is worked out by the database so that the pages cut
        // the whole list in one order. The id ends every order, so no card
        // can fall between two pages.
        if let Some(from) = near {
            query = query.order_by_with_nulls(km_from(from), Order::Asc, NullOrdering::Last);
        }

        let models = query
            .order_by_desc(workers::Column::UpdatedAt)
            .order_by_desc(workers::Column::Id)
            .offset(pagination.skip())
            .limit(*pagination.rows_per_page())
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let workers = models
            .into_iter()
            .map(Worker::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok((workers, count))
    }

    async fn delete(&self, id: i32) -> Result<bool, AppError> {
        let result = workers::Entity::delete_by_id(id)
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }
}
