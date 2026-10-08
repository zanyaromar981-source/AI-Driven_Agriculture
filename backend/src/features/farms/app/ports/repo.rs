use async_trait::async_trait;

use crate::{
    app::Pagination,
    features::farms::{
        app::AppError,
        domain::{Farm, FarmSummary},
    },
    shared::Phone,
};

#[async_trait]
pub trait FarmRepository: Send + Sync + std::fmt::Debug {
    /// Returns the page, and the total row count only on the first page. Later
    /// pages get `None`: the count is not recomputed, and the client is expected
    /// to have kept the one it was given.
    async fn find_all_by_owner(
        &self,
        owner: &Phone,
        pagination: &Pagination,
    ) -> Result<(Vec<FarmSummary>, Option<u64>), AppError>;

    /// Returns the farm with all of its cells.
    async fn find_by_id_and_owner(&self, id: i32, owner: &Phone) -> Result<Option<Farm>, AppError>;

    async fn count_by_owner(&self, owner: &Phone) -> Result<u64, AppError>;

    /// Creates a new entity with its cells. `entity.id()` must be `None`; the
    /// database assigns the ids.
    async fn create(&self, entity: &Farm) -> Result<Farm, AppError>;

    /// Updates an existing entity and the crop on its cells. `entity.id()`
    /// must be `Some`. The set of cells never changes after creation.
    async fn update(&self, entity: &Farm) -> Result<Farm, AppError>;

    async fn delete(&self, id: i32, owner: &Phone) -> Result<(), AppError>;
}
