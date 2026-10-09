use async_trait::async_trait;

use crate::{
    app::Pagination,
    features::workers::{
        app::AppError,
        domain::{CostIqd, CostPer, GeoPoint, Worker, WorkerSearch, ZoneSlug},
    },
    shared::Phone,
};

/// Which cards a list shows. Every part that is given must hold.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorkerFilter {
    pub zone: Option<ZoneSlug>,
    /// Looked for in the name and the note, whatever the case.
    pub search: Option<WorkerSearch>,
    /// Only cards that cost this much or less.
    pub max_cost: Option<CostIqd>,
    pub cost_per: Option<CostPer>,
    pub available: Option<bool>,
    /// Cards of these phones are left out.
    pub excluding: Vec<Phone>,
}

#[async_trait]
pub trait WorkerRepository: Send + Sync + std::fmt::Debug {
    /// Stores the card of its phone and returns it: created when the phone
    /// has none, every other column replaced when it has one. One statement
    /// on the unique phone, so two requests at the same moment leave one
    /// card.
    async fn save(&self, entity: &Worker) -> Result<Worker, AppError>;

    async fn find_by_phone(&self, phone: &Phone) -> Result<Option<Worker>, AppError>;

    /// Deletes the card of that phone. Returns whether there was one.
    async fn delete_by_phone(&self, phone: &Phone) -> Result<bool, AppError>;

    /// Returns one page of the cards the filter lets through, with how many
    /// there are in all: nearest to `near` first when it is given, cards
    /// without a point last; otherwise the newest update first.
    async fn find_page(
        &self,
        filter: &WorkerFilter,
        near: Option<&GeoPoint>,
        pagination: &Pagination,
    ) -> Result<(Vec<Worker>, u64), AppError>;

    /// Deletes the card with that id, whoever it belongs to. For Ministry
    /// staff. Returns whether there was one.
    async fn delete(&self, id: i32) -> Result<bool, AppError>;
}
