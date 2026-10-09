use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    app::Pagination,
    features::farms::{
        app::AppError,
        domain::{Farm, FarmLocation, FarmName, FarmSummary, IdempotencyKey, OwnedFarmSummary},
    },
    shared::Phone,
};

#[async_trait]
pub trait FarmRepository: Send + Sync + std::fmt::Debug {
    /// Returns every farm the owner has, oldest first. The list is short by
    /// construction: the number of farms per owner is capped.
    async fn find_all_by_owner(&self, owner: &Phone) -> Result<Vec<FarmSummary>, AppError>;

    /// Returns the farm with all of its cells.
    async fn find_by_id_and_owner(&self, id: i32, owner: &Phone) -> Result<Option<Farm>, AppError>;

    /// Returns the farm an earlier upload with the same key created.
    async fn find_by_idempotency_key_and_owner(
        &self,
        key: &IdempotencyKey,
        owner: &Phone,
    ) -> Result<Option<Farm>, AppError>;

    async fn count_by_owner(&self, owner: &Phone) -> Result<u64, AppError>;

    /// Whether a farm with that id is stored, whoever owns it.
    async fn exists(&self, id: i32) -> Result<bool, AppError>;

    /// Returns where every farm of every owner is, oldest first, without the
    /// owners.
    async fn find_all_locations(&self) -> Result<Vec<FarmLocation>, AppError>;

    /// Creates a new entity with its cells. `entity.id()` must be `None`; the
    /// database assigns the ids.
    async fn create(&self, entity: &Farm) -> Result<Farm, AppError>;

    /// Updates an existing entity and the crop on its cells. `entity.id()`
    /// must be `Some`. The set of cells never changes after creation.
    async fn update(&self, entity: &Farm) -> Result<Farm, AppError>;

    async fn delete(&self, id: i32, owner: &Phone) -> Result<(), AppError>;

    // The methods below are not scoped to an owner. They are for Ministry
    // staff on the dashboard; nothing a farmer's token reaches may call them.

    /// Returns one page of every owner's farms, newest first, or of one
    /// owner's when `owner` is given, with how many there are in all.
    async fn find_page(
        &self,
        owner: Option<&Phone>,
        pagination: &Pagination,
    ) -> Result<(Vec<OwnedFarmSummary>, u64), AppError>;

    /// Returns the farm with all of its cells, whoever owns it.
    async fn find_by_id(&self, id: i32) -> Result<Option<Farm>, AppError>;

    /// Gives the farm a new name in one statement and returns it, or `None`
    /// when no farm has that id. The outline and cells are not touched.
    async fn rename(
        &self,
        id: i32,
        name: &FarmName,
        now: DateTime<Utc>,
    ) -> Result<Option<Farm>, AppError>;

    /// Deletes the farm, whoever owns it. Returns whether there was one.
    async fn delete_by_id(&self, id: i32) -> Result<bool, AppError>;

    /// Deletes every farm of the owner, with their cells, in one statement.
    /// Returns how many farms were deleted.
    async fn delete_all_by_owner(&self, owner: &Phone) -> Result<u64, AppError>;
}
