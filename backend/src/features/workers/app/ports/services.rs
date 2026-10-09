use async_trait::async_trait;

use crate::{features::workers::app::AppError, shared::Phone};

/// Which accounts staff have blocked. Owned by the farmers feature: a
/// worker is an ordinary account there, and a card keeps only its phone.
#[async_trait]
pub trait AccountDirectory: Send + Sync + std::fmt::Debug {
    /// The phones of every blocked account.
    async fn blocked_phones(&self) -> Result<Vec<Phone>, AppError>;
}
