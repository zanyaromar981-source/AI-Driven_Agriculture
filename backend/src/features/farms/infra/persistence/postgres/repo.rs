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
            mappings::cell_active_model,
        },
    },
    shared::Phone,
};

/// Postgres accepts 65,535 bind parameters per statement. A cell insert binds
/// four, an id list binds one per cell.
const CELLS_PER_INSERT: usize = 10_000;
const IDS_PER_UPDATE: usize = 30_000;

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "farm repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(FromQueryResult)]
struct CellsPerCrop {
    farm_id: i32,
    crop: String,
    cells: i64,
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

    /// How many cells each of the farms has under each crop.
    async fn cells_per_crop(
        &self,
        farm_ids: Vec<i32>,
    ) -> Result<HashMap<i32, Vec<(String, usize)>>, AppError> {
        let counts = farm_cells::Entity::find()
            .select_only()
            .column(farm_cells::Column::FarmId)
            .column(farm_cells::Column::Crop)
            .column_as(farm_cells::Column::Id.count(), "cells")
            .filter(farm_cells::Column::FarmId.is_in(farm_ids))
            .group_by(farm_cells::Column::FarmId)
            .group_by(farm_cells::Column::Crop)
            .into_model::<CellsPerCrop>()
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let mut cells_per_crop: HashMap<i32, Vec<(String, usize)>> = HashMap::new();

        for count in counts {
            cells_per_crop
                .entry(count.farm_id)
                .or_default()
                .push((count.crop, usize::try_from(count.cells).unwrap_or_default()));
        }

        Ok(cells_per_crop)
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

        let mut cells_per_crop = self
            .cells_per_crop(models.iter().map(|model| model.id).collect())
            .await?;

        models
            .into_iter()
            .map(|model| {
                let cells = cells_per_crop.remove(&model.id).unwrap_or_default();

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

        for cells in entity.cells().chunks(CELLS_PER_INSERT) {
            farm_cells::Entity::insert_many(
                cells.iter().map(|cell| cell_active_model(model.id, cell)),
            )
            .exec(&transaction)
            .await
            .map_err(database_error)?;
        }

        let farm = Self::load(&transaction, model).await?;

        transaction.commit().await.map_err(database_error)?;

        Ok(farm)
    }

    async fn update(&self, entity: &Farm) -> Result<Farm, AppError> {
        if entity.id().is_none() {
            return Err(GlobalAppError::MissingValue(
                "Cannot update a farm that has not been persisted".to_string(),
            )
            .into());
        }

        let transaction = self.conn.begin().await.map_err(database_error)?;

        let model = farms::ActiveModel::from(entity)
            .update(&transaction)
            .await
            .map_err(|error| match error {
                // The farm was deleted between this request loading it and
                // writing it: it is gone, which is not a server fault.
                DbErr::RecordNotUpdated => GlobalAppError::NotFound.into(),
                other => database_error(other),
            })?;

        // Cells are never added or removed after creation, so an update only
        // has to move the repainted cells to the crop they now carry. Cells
        // this request did not change are not written, so a repaint running
        // at the same moment on other cells is not undone.
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

        let mut cells_per_crop = self
            .cells_per_crop(models.iter().map(|model| model.id).collect())
            .await?;

        let farms = models
            .into_iter()
            .map(|model| {
                let owner = Phone::new(model.phone.clone())?;
                let cells = cells_per_crop.remove(&model.id).unwrap_or_default();

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

fn cell_id(cell: &Cell) -> Result<i32, AppError> {
    cell.id().ok_or_else(|| {
        GlobalAppError::MissingValue(
            "Cannot update a farm cell that has not been persisted".to_string(),
        )
        .into()
    })
}
