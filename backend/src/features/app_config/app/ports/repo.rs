use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::features::app_config::{
    app::AppError,
    domain::{AppConfig, AppVersion},
};

#[async_trait]
pub trait AppConfigRepository: Send + Sync + std::fmt::Debug {
    /// Returns the one config row. It is seeded by the migration, so its
    /// absence is a fault, not an empty answer.
    async fn find(&self) -> Result<AppConfig, AppError>;

    /// Replaces every field of the one config row in one statement and
    /// returns it as stored.
    async fn replace(&self, config: &AppConfig) -> Result<AppConfig, AppError>;

    /// Notes that the farmer used this version at `now`, in one statement.
    /// A sighting of the same farmer and version later than `unless_after`
    /// is left as it is, so the row is written once an hour at most however
    /// many servers and restarts there are.
    async fn record_sighting(
        &self,
        farmer_id: i32,
        version: &AppVersion,
        now: DateTime<Utc>,
        unless_after: DateTime<Utc>,
    ) -> Result<(), AppError>;

    /// For each version, how many farmers were last seen on it, among the
    /// farmers seen since `since`. A farmer is counted once, under the
    /// version they used most recently.
    async fn count_farmers_by_version(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<(AppVersion, u64)>, AppError>;
}
