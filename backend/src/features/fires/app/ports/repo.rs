use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    app::Pagination,
    features::fires::{
        app::AppError,
        domain::{DetectionSpan, Fire, FireCorrection, FireStatus, ZoneSlug},
    },
};

/// Which stored fires the dashboard asks for.
#[derive(Clone, Debug, PartialEq)]
pub struct FireFilter {
    pub span: DetectionSpan,
    pub status: Option<FireStatus>,
    pub zone_slug: Option<ZoneSlug>,
}

#[async_trait]
pub trait FireRepository: Send + Sync + std::fmt::Debug {
    /// Returns every fire detected at or after `since`, newest first,
    /// whatever its status.
    async fn find_detected_since(&self, since: DateTime<Utc>) -> Result<Vec<Fire>, AppError>;

    /// Stores the fire under its external id, replacing the one already
    /// stored under that id if there is one. Returns the stored fire.
    async fn upsert(&self, entity: &Fire) -> Result<Fire, AppError>;

    /// Returns one page of the fires that match, newest first, and how many
    /// match in all.
    async fn find_page(
        &self,
        filter: &FireFilter,
        pagination: &Pagination,
    ) -> Result<(Vec<Fire>, u64), AppError>;

    async fn find_by_id(&self, id: i32) -> Result<Option<Fire>, AppError>;

    /// Stores a new fire. Returns `None`, having written nothing, when a
    /// fire is already stored under that external id.
    async fn create(&self, entity: &Fire) -> Result<Option<Fire>, AppError>;

    /// Replaces every field of the fire with that id except its external
    /// id, whatever is stored now. Returns `None` when there is no such fire.
    async fn update(&self, id: i32, correction: &FireCorrection) -> Result<Option<Fire>, AppError>;

    /// Removes the fire with that id. Removing one that is not there is not
    /// an error.
    async fn delete(&self, id: i32) -> Result<(), AppError>;
}
