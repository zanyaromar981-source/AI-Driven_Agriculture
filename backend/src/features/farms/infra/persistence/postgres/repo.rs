use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait,
    FromQueryResult, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
    sea_query::Expr,
};

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::farms::{
        app::{AppError, FarmRepository},
        domain::{
            Cell, Crop, Farm, FarmLocation, FarmName, FarmSummary, IdempotencyKey, OwnedFarmSummary,
        },
        infra::persistence::postgres::{
            entities::{farm_cells, farms},
            mappings::{cell_active_model, stored_outline},
        },
    },
    shared::Phone,
};

/// Postgres accepts 65,535 bind parameters per statement. A cell insert binds
/// five, an id list binds one per cell.
const CELLS_PER_INSERT: usize = 10_000;
const IDS_PER_UPDATE: usize = 30_000;

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "farm repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(FromQueryResult)]
struct InsidePerCrop {
    farm_id: i32,
    crop: String,
    inside_pct: f64,
}

#[derive(Debug)]
pub struct FarmPostgresRepository {
    conn: DatabaseConnection,
}

impl FarmPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    async fn load<C: ConnectionTrait>(conn: &C, model: farms::Model) -> Result<Farm, AppError> {
        let cells = farm_cells::Entity::find()
            .filter(farm_cells::Column::FarmId.eq(model.id))
            .order_by_asc(farm_cells::Column::Id)
            .all(conn)
            .await
            .map_err(database_error)?;

        Farm::try_from((model, cells))
    }

    /// For each of the farms, the summed `inside_pct` of the cells under
    /// each crop: the area of the crop, not a count of its cells.
    async fn inside_per_crop(
        &self,
        farm_ids: Vec<i32>,
    ) -> Result<HashMap<i32, Vec<(String, f64)>>, AppError> {
        let sums = farm_cells::Entity::find()
            .select_only()
            .column(farm_cells::Column::FarmId)
            .column(farm_cells::Column::Crop)
            .column_as(farm_cells::Column::InsidePct.sum(), "inside_pct")
            .filter(farm_cells::Column::FarmId.is_in(farm_ids))
            .group_by(farm_cells::Column::FarmId)
            .group_by(farm_cells::Column::Crop)
            .into_model::<InsidePerCrop>()
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let mut inside_per_crop: HashMap<i32, Vec<(String, f64)>> = HashMap::new();

        for sum in sums {
            inside_per_crop
                .entry(sum.farm_id)
                .or_default()
                .push((sum.crop, sum.inside_pct));
        }

        Ok(inside_per_crop)
    }

    async fn insert_cells<C: ConnectionTrait>(
        conn: &C,
        farm_id: i32,
        cells: &[Cell],
    ) -> Result<(), AppError> {
        for cells in cells.chunks(CELLS_PER_INSERT) {
            farm_cells::Entity::insert_many(
                cells.iter().map(|cell| cell_active_model(farm_id, cell)),
            )
            .exec(conn)
            .await
            .map_err(database_error)?;
        }

        Ok(())
    }
}

#[async_trait]
impl FarmRepository for FarmPostgresRepository {
    async fn find_all_by_owner(&self, owner: &Phone) -> Result<Vec<FarmSummary>, AppError> {
        let models = farms::Entity::find()
            .filter(farms::Column::Phone.eq(owner.as_str()))
            .order_by_asc(farms::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let mut inside_per_crop = self
            .inside_per_crop(models.iter().map(|model| model.id).collect())
            .await?;

        models
            .into_iter()
            .map(|model| {
                let cells = inside_per_crop.remove(&model.id).unwrap_or_default();

                FarmSummary::try_from((model, cells))
            })
            .collect()
    }

    async fn find_by_idempotency_key_and_owner(
        &self,
        key: &IdempotencyKey,
        owner: &Phone,
    ) -> Result<Option<Farm>, AppError> {
        let model = farms::Entity::find()
            .filter(farms::Column::Phone.eq(owner.as_str()))
            .filter(farms::Column::IdempotencyKey.eq(key.as_str()))
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        match model {
            Some(model) => Ok(Some(Self::load(&self.conn, model).await?)),
            None => Ok(None),
        }
    }

    async fn find_by_id_and_owner(&self, id: i32, owner: &Phone) -> Result<Option<Farm>, AppError> {
        let model = farms::Entity::find_by_id(id)
            .filter(farms::Column::Phone.eq(owner.as_str()))
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        match model {
            Some(model) => Ok(Some(Self::load(&self.conn, model).await?)),
            None => Ok(None),
        }
    }

    async fn count_by_owner(&self, owner: &Phone) -> Result<u64, AppError> {
        farms::Entity::find()
            .filter(farms::Column::Phone.eq(owner.as_str()))
            .count(&self.conn)
            .await
            .map_err(database_error)
    }

    async fn exists(&self, id: i32) -> Result<bool, AppError> {
        farms::Entity::find_by_id(id)
            .count(&self.conn)
            .await
            .map(|count| count > 0)
            .map_err(database_error)
    }

    async fn find_all_locations(&self) -> Result<Vec<FarmLocation>, AppError> {
        let models = farms::Entity::find()
            .order_by_asc(farms::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(FarmLocation::try_from).collect()
    }

    async fn create(&self, entity: &Farm) -> Result<Farm, AppError> {
        let transaction = self.conn.begin().await.map_err(database_error)?;

        let model = farms::ActiveModel::from(entity)
            .insert(&transaction)
            .await
            .map_err(database_error)?;

        Self::insert_cells(&transaction, model.id, entity.cells()).await?;

        let farm = Self::load(&transaction, model).await?;

        transaction.commit().await.map_err(database_error)?;

        Ok(farm)
    }

    async fn update(&self, entity: &Farm) -> Result<Farm, AppError> {
        let id = farm_id(entity)?;

        let transaction = self.conn.begin().await.map_err(database_error)?;

        // Only the time is written to the farm's row. The name and the
        // outline in `entity` are as old as its lookup, and writing them
        // back would undo an edit or a rename that landed since. The write
        // also takes the row's lock, so a repaint waits for an edit of the
        // same farm that is under way and never paints half of it.
        let touched = farms::Entity::update_many()
            .col_expr(
                farms::Column::UpdatedAt,
                Expr::value(entity.updated_at().naive_utc()),
            )
            .filter(farms::Column::Id.eq(id))
            .filter(farms::Column::Phone.eq(entity.owner().as_str()))
            .exec_with_returning(&transaction)
            .await
            .map_err(database_error)?;

        // No row: the farm was deleted between this request loading it and
        // writing it. It is gone, which is not a server fault.
        let Some(model) = touched.into_iter().next() else {
            return Err(GlobalAppError::NotFound.into());
        };

        // A repaint never adds or removes a cell, so it only has to move the
        // repainted cells to the crop they now carry. Cells this request did
        // not change are not written, so a repaint running at the same
        // moment on other cells is not undone. If an edit replaced the cells
        // since the lookup, these ids are gone and nothing is painted: the
        // result is the one of the repaint having come first.
        let mut ids_per_crop: HashMap<Crop, Vec<i32>> = HashMap::new();

        for cell in entity.cells().iter().filter(|cell| cell.repainted()) {
            let id = cell_id(cell)?;

            ids_per_crop.entry(cell.crop()).or_default().push(id);
        }

        for (crop, ids) in ids_per_crop {
            let crop = String::from(crop);

            for ids in ids.chunks(IDS_PER_UPDATE) {
                farm_cells::Entity::update_many()
                    .col_expr(farm_cells::Column::Crop, Expr::value(crop.clone()))
                    .filter(farm_cells::Column::FarmId.eq(model.id))
                    .filter(farm_cells::Column::Id.is_in(ids.to_vec()))
                    .filter(farm_cells::Column::Crop.ne(crop.clone()))
                    .exec(&transaction)
                    .await
                    .map_err(database_error)?;
            }
        }

        let farm = Self::load(&transaction, model).await?;

        transaction.commit().await.map_err(database_error)?;

        Ok(farm)
    }

    async fn replace(&self, entity: &Farm) -> Result<Farm, AppError> {
        let id = farm_id(entity)?;

        let transaction = self.conn.begin().await.map_err(database_error)?;

        // The lock is what makes the edit safe: a second edit or a repaint
        // of this farm waits here until this one has committed, so two sets
        // of cells are never mixed. Finding no row decides the 404, also
        // when the farm was deleted after the use case looked it up.
        let locked = farms::Entity::find_by_id(id)
            .filter(farms::Column::Phone.eq(entity.owner().as_str()))
            .lock_exclusive()
            .one(&transaction)
            .await
            .map_err(database_error)?;

        if locked.is_none() {
            return Err(GlobalAppError::NotFound.into());
        }

        // Only what an edit changes is written; the owner, the idempotency
        // key of the upload and the creation times stay as they are.
        let updated = farms::Entity::update_many()
            .col_expr(farms::Column::Name, Expr::value(entity.name().as_str()))
            .col_expr(
                farms::Column::Outline,
                Expr::value(stored_outline(entity.outline())),
            )
            .col_expr(
                farms::Column::UpdatedAt,
                Expr::value(entity.updated_at().naive_utc()),
            )
            .filter(farms::Column::Id.eq(id))
            .exec_with_returning(&transaction)
            .await
            .map_err(database_error)?;

        let Some(model) = updated.into_iter().next() else {
            return Err(GlobalAppError::NotFound.into());
        };

        // The whole set is deleted and written again rather than compared
        // with the old one: a moved border changes the share of nearly
        // every edge cell and the app sends every crop anyway, so a diff
        // would rewrite most rows in three kinds of statement to save
        // little, and after a plain delete no cell of the old outline can
        // be left behind.
        farm_cells::Entity::delete_many()
            .filter(farm_cells::Column::FarmId.eq(id))
            .exec(&transaction)
            .await
            .map_err(database_error)?;

        Self::insert_cells(&transaction, id, entity.cells()).await?;

        let farm = Self::load(&transaction, model).await?;

        transaction.commit().await.map_err(database_error)?;

        Ok(farm)
    }

    async fn delete(&self, id: i32, owner: &Phone) -> Result<(), AppError> {
        let result = farms::Entity::delete_many()
            .filter(farms::Column::Id.eq(id))
            .filter(farms::Column::Phone.eq(owner.as_str()))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        if result.rows_affected == 0 {
            return Err(GlobalAppError::NotFound.into());
        }

        Ok(())
    }

    async fn find_page(
        &self,
        owner: Option<&Phone>,
        pagination: &Pagination,
    ) -> Result<(Vec<OwnedFarmSummary>, u64), AppError> {
        let mut query = farms::Entity::find();

        if let Some(owner) = owner {
            query = query.filter(farms::Column::Phone.eq(owner.as_str()));
        }

        let count = query
            .clone()
            .count(&self.conn)
            .await
            .map_err(database_error)?;

        let models = query
            .order_by_desc(farms::Column::CreatedAt)
            .order_by_desc(farms::Column::Id)
            .offset(pagination.skip())
            .limit(*pagination.rows_per_page())
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let mut inside_per_crop = self
            .inside_per_crop(models.iter().map(|model| model.id).collect())
            .await?;

        let farms = models
            .into_iter()
            .map(|model| {
                let owner = Phone::new(model.phone.clone())?;
                let cells = inside_per_crop.remove(&model.id).unwrap_or_default();

                Ok(OwnedFarmSummary::new(
                    owner,
                    FarmSummary::try_from((model, cells))?,
                ))
            })
            .collect::<Result<Vec<_>, AppError>>()?;

        Ok((farms, count))
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Farm>, AppError> {
        let model = farms::Entity::find_by_id(id)
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        match model {
            Some(model) => Ok(Some(Self::load(&self.conn, model).await?)),
            None => Ok(None),
        }
    }

    async fn rename(
        &self,
        id: i32,
        name: &FarmName,
        now: DateTime<Utc>,
    ) -> Result<Option<Farm>, AppError> {
        // One statement decides whether there is a farm to rename, and it
        // writes the name only: a repaint by the farmer at the same moment
        // is not undone.
        let renamed = farms::Entity::update_many()
            .col_expr(farms::Column::Name, Expr::value(name.as_str()))
            .col_expr(farms::Column::UpdatedAt, Expr::value(now.naive_utc()))
            .filter(farms::Column::Id.eq(id))
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        match renamed.into_iter().next() {
            Some(model) => Ok(Some(Self::load(&self.conn, model).await?)),
            None => Ok(None),
        }
    }

    async fn delete_by_id(&self, id: i32) -> Result<bool, AppError> {
        let result = farms::Entity::delete_many()
            .filter(farms::Column::Id.eq(id))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }

    async fn delete_all_by_owner(&self, owner: &Phone) -> Result<u64, AppError> {
        // The cells go with their farms: the foreign key cascades.
        let result = farms::Entity::delete_many()
            .filter(farms::Column::Phone.eq(owner.as_str()))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected)
    }
}

fn farm_id(farm: &Farm) -> Result<i32, AppError> {
    farm.id().ok_or_else(|| {
        GlobalAppError::MissingValue("Cannot update a farm that has not been persisted".to_string())
            .into()
    })
}

fn cell_id(cell: &Cell) -> Result<i32, AppError> {
    cell.id().ok_or_else(|| {
        GlobalAppError::MissingValue(
            "Cannot update a farm cell that has not been persisted".to_string(),
        )
        .into()
    })
}
