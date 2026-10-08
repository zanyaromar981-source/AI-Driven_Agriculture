use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::features::fires::{app::AppError, domain::Fire};

#[async_trait]
pub trait FireRepository: Send + Sync + std::fmt::Debug {
    /// Returns every fire detected at or after `since`, newest first,
    /// whatever its status.
    async fn find_detected_since(&self, since: DateTime<Utc>) -> Result<Vec<Fire>, AppError>;

    /// Stores the fire under its external id, replacing the one already
    /// stored under that id if there is one. Returns the stored fire.
    async fn upsert(&self, entity: &Fire) -> Result<Fire, AppError>;
}
