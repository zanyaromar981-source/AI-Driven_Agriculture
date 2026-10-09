use async_trait::async_trait;

use crate::features::versions::{app::AppError, domain::DataVersion};

#[async_trait]
pub trait VersionRepository: Send + Sync + std::fmt::Debug {
    /// Every topic's current version. The versions are raised by the
    /// database itself whenever a table of the topic is written, inside the
    /// same transaction as the write.
    async fn find_all(&self) -> Result<Vec<DataVersion>, AppError>;
}
